use std::env;

use axum::{extract::Query, response::Redirect, Json};
use oauth2::{
    basic::{BasicClient, BasicErrorResponseType, BasicTokenType},
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields,
    EndpointNotSet, EndpointSet, RedirectUrl, RevocationErrorResponseType, Scope,
    StandardErrorResponse, StandardRevocableToken, StandardTokenIntrospectionResponse,
    StandardTokenResponse, TokenResponse, TokenUrl,
};
use reqwest::{ClientBuilder, StatusCode};
use serde::Deserialize;

use crate::token::{get_token_exp, Claims};

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
) -> Result<Json<String>, (StatusCode, &'static str)> {
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
                    let exp = get_token_exp();
                    let email = user_info
                        .get("email")
                        .map(|email| email.as_str().map(|email| email.to_owned()))
                        .flatten();

                    let claims = Claims {
                        sub: None,
                        exp,
                        email,
                    };
                    Ok(Json(serde_json::to_string(&claims).map_err(|err| {
                        eprintln!("Error serializing claims: {}", err);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Error serializing claims",
                        )
                    })?))
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
