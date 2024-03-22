use std::env;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router,
};
use dotenv::dotenv;
use my_hood_server::{config::Config, graphql::get_schema};
use sqlx::PgPool;
use tokio::net::TcpListener;

mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;
    env_logger::init();

    let db_url = env::var("DATABASE_URL").unwrap();
    let db = PgPool::connect(&db_url).await.unwrap();

    let host = env::var("HOST").unwrap();
    let port = env::var("PORT").unwrap();

    let config = Config::new();

    let schema = get_schema(db.clone(), config.clone());

    async fn graphql_playground() -> impl IntoResponse {
        response::Html(GraphiQLSource::build().endpoint("/").finish())
    }

    let app = Router::new().route(
        "/",
        get(graphql_playground).post_service(GraphQL::new(schema)),
    );
    println!("Serving on http://{host}:{port}");
    axum::serve(
        TcpListener::bind(format!("{host}:{port}")).await.unwrap(),
        app,
    )
    .await
    .expect("Error spawning server");
    Ok(())
}
