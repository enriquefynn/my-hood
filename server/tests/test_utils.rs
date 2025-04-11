use std::{env, thread};

use async_graphql::{EmptySubscription, Schema};
use axum::{
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDate;
use dotenv::dotenv;
use futures::future::join_all;
use my_hood_server::{
    config::Config,
    graphql::{Mutation, Query},
    token::Claims,
    user::model::User,
    DB,
};
use reqwest::Url;
use sqlx::Executor;
use uuid::Uuid;

pub struct TestDatabase {
    pub pool: DB,
    pub db_name: String,
    pub admin: User,
    pub admin_url: String,
}

impl TestDatabase {
    /// Asynchronously creates a new test database and returns a guard containing the pool.
    pub async fn new() -> Self {
        dotenv().unwrap();
        let db_url = env::var("DATABASE_URL").unwrap();
        let admin_url = db_url.clone();
        let db = DB::connect(&db_url).await.unwrap();

        let db_name = format!("test_{}", Uuid::new_v4());

        let create_db_query = format!("CREATE DATABASE \"{}\"", db_name);
        db.execute(create_db_query.as_str()).await.unwrap();

        let mut url = Url::parse(&db_url).expect("Failed to parse DATABASE_URL");
        url.set_path(&db_name);

        let db_url = url.to_string();

        env::set_var("DATABASE_URL", &db_url);
        let pool = DB::connect(&db_url)
            .await
            .expect("Failed to connect to test database");

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let mut tx = pool.begin().await.unwrap();
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
        TestDatabase {
            pool,
            db_name,
            admin: user,
            admin_url,
        }
    }

    pub fn get_schema_for_tests(
        &self,
        config: Config,
        claims: Claims,
    ) -> Schema<Query, Mutation, EmptySubscription> {
        Schema::build(Query::default(), Mutation::default(), EmptySubscription)
            .data(self.pool.clone())
            .data(config)
            .data(claims)
            .finish()
    }

    /// Creates an association with the given number of admin, member, and
    /// treasurer users.  First `num_member` users are created as members, and
    /// `num_treasurer` users are created as treasurers.  Each user is created
    /// with the specified number of fields.
    pub async fn create_association_admin_member_treasury_fields(
        &self,
        num_member: u32,
        num_treasurer: u32,
        num_fields: u32,
    ) {
        let config = Config::new();

        let claims = Claims {
            sub: Some(self.admin.id),
            exp: 0,
            email: self.admin.email.clone(),
        };
        let schema = self.get_schema_for_tests(config.clone(), claims);

        let all_users = num_member + num_treasurer;

        let users_json = create_users_json(all_users);

        let user_id_futures = users_json
            .iter()
            .map(async |user| {
                let request = async_graphql::Request::new(user.to_string());
                let response = schema.execute(request).await;

                if response.is_err() {
                    panic!("Error executing request: {:?}", response);
                }

                let response = response
                    .data
                    .into_json()
                    .expect("Something went wrong parsing the response");
                let user_id = response["createUser"]["id"]
                    .as_str()
                    .expect("Should get id from user");
                let user_id = Uuid::parse_str(user_id).expect("Should parse id to Uuid");
                user_id
            })
            .collect::<Vec<_>>();
        let user_ids: Vec<Uuid> = join_all(user_id_futures).await;

        let create_association_mutation = format!(
            r#"mutation {{
            createAssociation(association: {{
                name: "foo",
                neighborhood: "FOOO",
                country: "BR",
                state: "BA",
                address: "Foobar street",
            }})
            {{
            id
            }}
            }}"#,
        );
        let request = async_graphql::Request::new(create_association_mutation.to_string());
        let response = schema
            .execute(request)
            .await
            .data
            .into_json()
            .expect("Something went wrong parsing the response");
        let association_id = response["createAssociation"]["id"]
            .as_str()
            .expect("Should get id from association");
        let association_id = Uuid::parse_str(association_id).expect("Should parse id to Uuid");

        let user_memberships = create_user_membership(user_ids, association_id);
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Clone values for the cleanup.
        let admin_url = self.admin_url.clone();
        let db_name = self.db_name.clone();

        // Spawn a new thread and runtime to run async cleanup.
        // This ensures that cleanup happens even if the test exits unexpectedly.
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = cleanup_test_db(&admin_url, &db_name).await {
                    eprintln!("Error cleaning up test database {}: {}", db_name, e);
                }
            });
        })
        .join()
        .expect("Cleanup thread panicked");
    }
}

pub async fn cleanup_test_db(admin_url: &str, db_name: &str) -> Result<(), sqlx::Error> {
    let conn = DB::connect(admin_url).await?;
    let terminate_query = format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
        db_name
    );
    conn.execute(terminate_query.as_str()).await?;
    let drop_query = format!("DROP DATABASE \"{}\"", db_name);
    conn.execute(drop_query.as_str()).await?;
    Ok(())
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

pub fn create_users_json(n_users: u32) -> Vec<String> {
    (0..n_users)
        .into_iter()
        .map(|id| {
            format!(
                r#"mutation {{
            createUser(userInput: {{
                name: "Test User {}",
                email: "test{}@gmail.com",
                birthday: "2012-11-19",
                address: "Rua A nr 1",
                usesWhatsapp: true
            }})
        {{
        id
        }}
        }}"#,
                id, id
            )
        })
        .collect::<Vec<String>>()
}

pub fn create_user_membership(user_ids: Vec<Uuid>, association_id: Uuid) -> Vec<serde_json::Value> {
    user_ids
        .into_iter()
        .map(|user_id| {
            serde_json::json!({
                "query": format!("mutation {{
                createUserAssociation(userId: \"{}\", associationId: \"{}\")
            }}", user_id, association_id)
            })
        })
        .collect()
}
