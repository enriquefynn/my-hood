use std::env;

use actix_web::{
    guard,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use dotenv::dotenv;
use server::user::{get_user_schema, graphql::ProjectSchema};
use sqlx::PgPool;

mod utils;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").unwrap();
    let db = PgPool::connect(&db_url).await.unwrap();

    // Graphql entry.
    async fn index(schema: Data<ProjectSchema>, req: GraphQLRequest) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }

    let user_schema_data = get_user_schema(db.clone());

    async fn graphql_playground() -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(playground_source(GraphQLPlaygroundConfig::new("/")))
    }

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(user_schema_data.clone()))
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
