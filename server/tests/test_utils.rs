use std::env;

use async_graphql::{EmptySubscription, Schema};
use axum::{
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDate;
use dotenv::dotenv;
use my_hood_server::{
    config::Config,
    graphql::{Mutation, Query},
    token::Claims,
    user::model::User,
    DB,
};

pub async fn setup_db() -> (DB, User) {
    dotenv().unwrap();
    let db_url = env::var("DATABASE_URL").unwrap();
    let db = DB::connect(&db_url).await.unwrap();
    let mut tx = db.begin().await.unwrap();

    sqlx::query!("DROP SCHEMA IF EXISTS public CASCADE")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query!("CREATE SCHEMA public")
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&mut *tx).await.unwrap();

    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO "User" (name, birthday, address, email) VALUES
        ($1,$2,$3,$4) RETURNING *
        "#,
        "Test User 1",
        NaiveDate::from_ymd_opt(2012, 11, 19).unwrap(),
        "Rua A nr 1",
        "testuser1@test.com"
    )
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    (db, user)
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route(
            "/json",
            post(|payload: Json<serde_json::Value>| async move {
                Json(serde_json::json!({ "data": payload.0 }))
            }),
        )
        .route("/requires-connect-info", get(|| async move {}))
}

pub fn get_users_json(n_users: u32) -> Vec<serde_json::Value> {
    (0..n_users)
        .into_iter()
        .map(|id| {
            serde_json::json!({
                "query": format!("mutation {{
                    createUser(user:
                        {{
                            name: \"Test User {id}\",
                            email: \"test_user{id}@gmail.com\",
                            birthday: \"2012-11-19\",
                            address: \"Rua A nr 1\",
                            usesWhatsapp: true
                        }}
                    ) {{id}}
                }}")
            })
        })
        .collect::<Vec<_>>()
}

// fn get_id(
//     json_data: async_graphql_value::Value,
//     method_name: &str,
//     key_name: &str,
// ) -> async_graphql_value::Value {
//     let value = match json_data {
//         async_graphql_value::Value::Object(obj) => obj["data"].clone(),
//         _ => panic!(),
//     };

//     let value = match value {
//         async_graphql_value::Value::Object(obj) => obj[method_name].clone(),
//         _ => panic!(),
//     };
//     let val = match value {
//         async_graphql_value::Value::Object(obj) => obj[key_name].clone(),
//         _ => panic!(),
//     };
//     val
// }

pub fn get_schema_for_tests(
    db: DB,
    config: Config,
    claims: Claims,
) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db)
        .data(config)
        .data(claims)
        .finish()
}
