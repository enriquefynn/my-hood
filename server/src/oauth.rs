use std::env;

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    Extension,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    EndpointNotSet, EndpointSet, RedirectUrl, RevocationErrorResponseType, Scope,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use reqwest::{ClientBuilder, StatusCode};
use serde::Deserialize;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::{
    token::{get_token_exp, Claims},
    user::model::User,
    DB,
};

#[derive(Debug, Deserialize)]
pub struct OAuthRequest {
    code: String,
    #[allow(dead_code)]
    state: String,
}

fn get_oauth_client() -> Client<
    StandardErrorResponse<BasicErrorResponseType>,
    StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, BasicTokenType>,
    StandardRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
> {
    let client_id = ClientId::new(env::var("GOOGLE_OAUTH_CLIENT_ID").expect("Missing client ID"));
    let client_secret =
        ClientSecret::new(env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("Missing client secret"));
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/auth".to_string())
        .expect("Invalid authorization URL");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid token URL");
    let rediret_url = RedirectUrl::new(
        env::var("GOOGLE_OAUTH_REDIRECT_URL").expect("Missing oauth redirect url"),
    )
    .expect("Invalid redirect URL");

    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(rediret_url)
}

pub async fn google_oauth_client() -> Redirect {
    let client = get_oauth_client();

    let (auth_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_owned()))
        .url();
    Redirect::to(auth_url.as_ref())
}

/// Handler to receive the callback from the OAuth provider.
pub async fn callback_handler(
    Query(params): Query<OAuthRequest>,
    db: Extension<DB>,
    cookies: Cookies,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    let client = get_oauth_client();

    // TODO: Verify that `params.state` matches the previously stored CSRF token.

    let http_client = ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Exchange the authorization code for an access token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(&http_client)
        .await;

    match token_result {
        Ok(token) => {
            let userinfo_endpoint = "https://www.googleapis.com/oauth2/v3/userinfo";
            let access_token = token.access_token().secret();
            let client_reqwest = reqwest::Client::new();
            let response = client_reqwest
                .get(userinfo_endpoint)
                .bearer_auth(access_token)
                .send()
                .await;
            match response {
                Ok(response) => {
                    let user_info: serde_json::Value = response.json().await.map_err(|_| {
                        (StatusCode::INTERNAL_SERVER_ERROR, "Error parsing user info")
                    })?;

                    // Craft a new JWT token so user can create an account.
                    let email_opt = user_info
                        .get("email")
                        .map(|email| email.as_str().map(|email| email.to_owned()))
                        .flatten();

                    let (id, email) = if let Some(email) = &email_opt {
                        let user = User::read_one_by_email(&db, email).await.map_err(|_| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Error reading user from DB",
                            )
                        })?;
                        match user {
                            Some(user) => (Some(user.id), user.email),
                            None => (None, Some(email.clone())),
                        }
                    } else {
                        (None, None)
                    };

                    let token = get_token(id, email)?;
                    let mut auth_cookie = Cookie::new("auth_token", token.clone());
                    auth_cookie.set_path("/");
                    auth_cookie.set_http_only(false);
                    cookies.add(auth_cookie);

                    if id.is_some() {
                        // User exists, redirect to the calendar
                        Ok(Redirect::to("http://localhost:3000/calendar"))
                    } else {
                        Ok(Redirect::to("http://localhost:3000/user/register"))
                    }
                }
                Err(err) => {
                    eprintln!("Error obtaining user info: {}", err);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Error obtaining user info",
                    ))
                }
            }
        }
        Err(err) => {
            eprintln!("Error obtaining token: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Error obtaining token"))
        }
    }
}

pub fn get_token(
    sub: Option<Uuid>,
    email: Option<String>,
) -> Result<String, (StatusCode, &'static str)> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let exp = get_token_exp();
    let claims = Claims {
        sub,
        exp,
        email: email,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token creation failed"))?;
    Ok(token)
}
