use actix_web::{
    guard,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use db::Database;
use graphql::{Mutation, Query};
use sqlx::SqlitePool;

mod db;
mod graphql;
mod schema;
mod utils;

const DB_URL: &str = "sqlite://sqlite.db";

pub type ProjectSchema = Schema<Query, Mutation, EmptySubscription>;
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let db = SqlitePool::connect(DB_URL).await.unwrap();
    let database = Database { db };

    // Graphql entry.
    async fn index(schema: Data<ProjectSchema>, req: GraphQLRequest) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    let schema_data = Schema::build(Query, Mutation, EmptySubscription)
        .data(database)
        .finish();

    async fn graphql_playground() -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    }

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema_data.clone()))
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(
                web::resource("/")
                    .guard(guard::Get())
                    .to(graphql_playground),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    Ok(())
}
