use std::env;

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
use dotenv::dotenv;
use server::graphql::{get_schema, Mutation, Query};
use sqlx::PgPool;

mod utils;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap();
    let db = PgPool::connect(&db_url).await.unwrap();

    // Graphql entry.
    async fn index(
        schema: Data<Schema<Query, Mutation, EmptySubscription>>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    // let user_schema_data = get_user_schema(db.clone());
    // let association_schema = get_association_schema(db.clone());
    let schema = get_schema(db.clone());

    async fn graphql_playground() -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    }

    HttpServer::new(move || {
        App::new()
            // .app_data(Data::new(user_schema_data.clone()))
            // .app_data(Data::new(association_schema.clone()))
            .app_data(Data::new(schema.clone()))
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
