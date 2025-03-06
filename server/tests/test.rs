#[cfg(test)]
mod tests {
    use std::env;

    use axum::{
        body::Body,
        extract::ConnectInfo,
        http::{self, Request},
        routing::{get, post},
        Json, Router,
    };
    use dotenv::dotenv;
    use my_hood_server::{config::Config, graphql::get_schema, DB};
    use serde_json::{json, Value};
    use tokio::net::unix::SocketAddr;
    use tower::{Service, ServiceExt}; // for `call`, `oneshot`, and `ready`

    use super::*;

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

    async fn setup_db() -> DB {
        dotenv().unwrap();
        let db_url = env::var("DATABASE_URL").unwrap();
        let db = DB::connect(&db_url).await.unwrap();
        sqlx::query!("DROP SCHEMA IF EXISTS public CASCADE")
            .execute(&db)
            .await
            .unwrap();
        sqlx::query!("CREATE SCHEMA public")
            .execute(&db)
            .await
            .unwrap();
        sqlx::migrate!("./migrations").run(&db).await.unwrap();
        db
    }

    fn get_id(
        json_data: async_graphql_value::Value,
        method_name: &str,
        key_name: &str,
    ) -> async_graphql_value::Value {
        let value = match json_data {
            async_graphql_value::Value::Object(obj) => obj["data"].clone(),
            _ => panic!(),
        };

        let value = match value {
            async_graphql_value::Value::Object(obj) => obj[method_name].clone(),
            _ => panic!(),
        };
        let val = match value {
            async_graphql_value::Value::Object(obj) => obj[key_name].clone(),
            _ => panic!(),
        };
        val
    }

    fn app() -> Router {
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

    #[tokio::test]
    async fn test_create_user() {
        let db = setup_db().await;
        let config = Config::new();

        let schema = get_schema(db.clone(), config.clone());

        // let app = app();

        let create_user_mutation = r#"mutation {
            createUser(user: {
                name: "Test User",
                email: "test@gmail.com",
                birthday: "2012-11-19",
                address: "Rua A nr 1",
                usesWhatsapp: true
            }) {
                name,
                email,
                birthday,
                address,
                usesWhatsapp,
            }
        }
        "#;

        let request = async_graphql::Request::new(create_user_mutation.to_string());
        let response = schema.execute(request).await.data.into_value();

        let expected_response = serde_json::from_str(
            r#"{
                "createUser": {
                    "name": "Test User",
                    "email": "test@gmail.com",
                    "birthday": "2012-11-19",
                    "address": "Rua A nr 1",
                    "usesWhatsapp": true
                }
            }"#,
        )
        .unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_get_user() {
        let db = setup_db().await;
        let config = Config::new();

        let schema = get_schema(db.clone(), config.clone());

        let create_user_mutation = r#"mutation {
            createUser(user: {
                name: "Test User",
                email: "test@gmail.com",
                birthday: "2012-11-19",
                address: "Rua A nr 1",
                usesWhatsapp: true
            }) {
                id
            }
        }
        "#;

        let request = async_graphql::Request::new(create_user_mutation.to_string());
        let response = schema.execute(request).await.data.into_value();
        let user = response.into_json().unwrap();
        let user_id = user
            .get("createUser")
            .unwrap()
            .get("id")
            .unwrap()
            .as_str()
            .unwrap();

        let get_user_query = format!(
            r#"query {{
                user(id: "{}") {{
                    name,
                    email,
                    birthday,
                    address,
                    usesWhatsapp,
                }}
            }}"#,
            user_id
        );
        let request = async_graphql::Request::new(get_user_query);
        let response = schema.execute(request).await.data.into_value();

        let expected_response = serde_json::from_str(
            r#"{
                "user": {
                    "name": "Test User",
                    "email": "test@gmail.com",
                    "birthday": "2012-11-19",
                    "address": "Rua A nr 1",
                    "usesWhatsapp": true
                }
            }"#,
        )
        .unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_create_association() {
        let db = setup_db().await;
        let config = Config::new();

        let schema = get_schema(db.clone(), config.clone());

        let create_association_mutation = r#"mutation {
            createAssociation(association: {
                name: "Foo"
                neighborhood: "Bar"
                country: "BR"
                state: "BA"
                address: "Rua A nr. 2"
            }) {
              name,
              neighborhood,
              country,
              state,
              address
            }
          }
        "#;

        let request = async_graphql::Request::new(create_association_mutation.to_string());
        let response = schema.execute(request).await.data.into_value();

        let expected_response = serde_json::from_str(
            r#"{
                "createAssociation": {
                    "name": "Foo",
                    "neighborhood": "Bar",
                    "country": "BR",
                    "state": "BA",
                    "address": "Rua A nr. 2"
                }
            }"#,
        )
        .unwrap();
        assert_eq!(response, expected_response);
    }

    #[tokio::test]
    async fn test_users_association() {
        let db = setup_db().await;
        let config = Config::new();

        let schema = get_schema(db.clone(), config.clone());

        let users = get_users_json(100);
        for create_user in users {
            let request = async_graphql::Request::new(create_user.to_string());
            let response = schema.execute(request).await.data.into_value();
        }
    }
}
