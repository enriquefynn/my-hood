use std::env;

use actix_web::{
    get, guard,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use dotenv::dotenv;
use server::{
    config::Config,
    graphql::{get_schema, Mutation, Query},
    oauth::google_oauth_handler,
};
use sqlx::PgPool;

mod utils;

async fn oauth_redirect() -> impl Responder {
    let google_auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=email%20profile",
        env::var("CLIENT_ID").unwrap(),
        "http://localhost:8000/oauth/callback"
    );
    HttpResponse::Found()
        .append_header(("location", google_auth_url.as_str()))
        .finish()
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();
    let db = PgPool::connect(&db_url).await.unwrap();

    let host = env::var("HOST").unwrap();
    let port = env::var("PORT").unwrap();

    let config = Config::new();

    // Graphql entry.
    async fn index(
        schema: Data<Schema<Query, Mutation, EmptySubscription>>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    let schema = get_schema(db.clone(), config.clone());

    async fn graphql_playground() -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    }

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .app_data(Data::new(db.clone()))
            .app_data(Data::new(config.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(google_oauth_handler)
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .to(graphql_playground),
            )
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await?;

    Ok(())
}
