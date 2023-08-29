use actix_web::{cookie::Cookie, get, web, HttpResponse, Responder};
use jsonwebtoken::Header;
use serde::Deserialize;

use crate::{
    config::Config,
    error::HoodError,
    token::TokenClaims,
    user::model::{User, UserInput},
    DB,
};

use reqwest::{header::LOCATION, Client, Url};
use std::error::Error;

#[derive(Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub id_token: String,
}

#[derive(Debug, Deserialize)]
pub struct GoogleUser {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: String,
    pub given_name: String,
    pub family_name: String,
    pub picture: String,
    pub locale: String,
}

pub async fn request_token(
    authorization_code: &str,
    data: &Config,
) -> Result<OAuthResponse, Box<dyn Error>> {
    let redirect_url = data.google_oauth_redirect_url.to_owned();
    let client_secret = data.google_oauth_client_secret.to_owned();
    let client_id = data.google_oauth_client_id.to_owned();

    let root_url = "https://oauth2.googleapis.com/token";
    let client = Client::new();

    let params = [
        ("grant_type", "authorization_code"),
        ("redirect_uri", redirect_url.as_str()),
        ("client_id", client_id.as_str()),
        ("code", authorization_code),
        ("client_secret", client_secret.as_str()),
    ];
    let response = client.post(root_url).form(&params).send().await?;

    if response.status().is_success() {
        let oauth_response = response.json::<OAuthResponse>().await?;
        Ok(oauth_response)
    } else {
        let message = "An error occurred while trying to retrieve access token.";
        Err(From::from(message))
    }
}

pub async fn get_google_user(
    access_token: &str,
    id_token: &str,
) -> Result<GoogleUser, Box<dyn Error>> {
    let client = Client::new();
    let mut url = Url::parse("https://www.googleapis.com/oauth2/v1/userinfo").unwrap();
    url.query_pairs_mut().append_pair("alt", "json");
    url.query_pairs_mut()
        .append_pair("access_token", access_token);

    let response = client.get(url).bearer_auth(id_token).send().await?;

    if response.status().is_success() {
        let user_info = response.json::<GoogleUser>().await?;
        Ok(user_info)
    } else {
        let message = "An error occurred while trying to retrieve user information.";
        Err(From::from(message))
    }
}

#[derive(Debug, Deserialize)]
pub struct QueryCode {
    pub code: String,
    pub state: String,
}

#[get("/oauth/google/callback")]
pub async fn google_oauth_handler(
    query: web::Query<QueryCode>,
    config: web::Data<Config>,
    db: web::Data<DB>,
) -> Result<impl Responder, HoodError> {
    let code = &query.code;
    let state = &query.state;

    if code.is_empty() {
        return Ok(HttpResponse::Unauthorized().json(
            serde_json::json!({"status": "fail", "message": "Authorization code not provided!"}),
        ));
    }

    let token_response = request_token(code.as_str(), &*config)
        .await
        .map_err(|err| HoodError {
            msg: err.to_string(),
        })?;

    let google_user = get_google_user(&token_response.access_token, &token_response.id_token)
        .await
        .map_err(|err| HoodError {
            msg: err.to_string(),
        })?;

    let user = User::read_one_by_email(&db, &google_user.email)
        .await
        .map_err(|err| HoodError {
            msg: err.to_string(),
        })?;

    let user = match user {
        Some(user) => {
            // User already exists, signing in as the user.
            user
        }
        None => {
            // We need to create a new user.
            let mut user_input: UserInput =
                serde_json::from_str(state).map_err(|err| HoodError {
                    msg: err.to_string(),
                })?;

            // We reuse the `state` field of the OAuth method to encode extra
            // fields needed for user creation.
            user_input.email = Some(google_user.email);
            user_input.name = Some(google_user.name);
            user_input.profile_url = Some(google_user.picture);

            User::create(&db, user_input)
                .await
                .map_err(|err| HoodError {
                    msg: err.to_string(),
                })?
        }
    };

    let jwt_secret = config.jwt_secret.to_owned();
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(config.jwt_max_age)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id,
        exp,
        iat,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build("token", token)
        .path("/")
        .max_age(actix_web::cookie::time::Duration::new(
            60 * config.jwt_max_age,
            0,
        ))
        .http_only(true)
        .finish();

    // let frontend_origin = config.client_origin.to_owned();
    let mut response = HttpResponse::Found();
    // response.append_header((LOCATION, format!("{}{}", frontend_origin, state)));
    response.cookie(cookie);
    Ok(response.finish())
}
