use std::{env, sync::Arc, thread};

use async_graphql::{EmptySubscription, Schema};
use async_trait::async_trait;
use axum::{
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use dotenv::dotenv;
use futures::future::join_all;
use jsonwebtoken::{encode, EncodingKey, Header};
use my_hood_server::{
    association::model::Association,
    config::Config,
    field::model::Field,
    graphql::{Mutation, Query},
    token::Claims,
    user::model::{User, UserInput},
    Clock, DB,
};
use reqwest::Url;
use sqlx::Executor;
use uuid::Uuid;

pub struct TestDatabase {
    pub pool: DB,
    pub db_name: String,
    pub admin: User,
    pub admin_url: String,
    pub clock: Arc<dyn Clock>,
}

pub struct TestAssociationUsers {
    pub association: Association,
    pub members: Vec<User>,
    pub treasurers: Vec<User>,
    pub fields: Vec<Field>,
}

impl TestDatabase {
    async fn get_claim_for_user(user_id: Uuid, email: String) -> String {
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let claims = Claims {
            sub: Some(user_id),
            exp: 0,
            email: Some(email),
        };
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("Failed to encode token");
        token
    }

    pub async fn create_logins(&self, num_users: u32) -> Vec<User> {
        let users = (0..num_users)
            .map(async |i| {
                let name = format!("Test User {}", i);
                let birthday: NaiveDate = "2012-11-19".parse().unwrap();
                let address = "Rua A nr 1".to_string();
                let email = format!("testuser{}@test.com", i);
                let password_hash = "password".to_string();

                User::create(
                    &self.pool,
                    UserInput {
                        name: Some(name.clone()),
                        birthday: birthday.clone(),
                        address: address.clone(),
                        email: Some(email.clone()),
                        password_hash: Some(password_hash),
                        uses_whatsapp: true,
                        identities: None,
                        personal_phone: None,
                        commercial_phone: None,
                        activity: None,
                        profile_url: None,
                    },
                ) //name, birthday, address, email, password)
                .await
                .unwrap()
            })
            .collect::<Vec<_>>();
        let all = join_all(users).await;
        return all;
    }

    pub async fn new(now: DateTime<Utc>) -> Self {
        dotenv().unwrap();
        let db_url = env::var("DATABASE_URL").unwrap();
        let admin_url = db_url.clone();
        let db = DB::connect(&db_url).await.unwrap();

        let db_name = format!("test_{}", Uuid::new_v4());

        let create_db_query = format!("CREATE DATABASE \"{}\"", db_name);
        db.execute(create_db_query.as_str()).await.unwrap();

        let mut url = Url::parse(&db_url).expect("Failed to parse DATABASE_URL");
        url.set_path(&db_name);
        println!("Creating test database: {}", url);

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
            "default_user@test.com"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let clock = Arc::new(FixedClock(now));
        TestDatabase {
            pool,
            db_name,
            admin: user,
            admin_url,
            clock,
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
            .data(self.clock.clone())
            .finish()
    }

    /// Creates an association with the given number of admin, member, and
    /// treasurer users.  First `num_member` users are created as members, and
    /// `num_treasurer` users are created as treasurers.
    pub async fn create_association_admin_member_treasury_fields(
        &self,
        n_member: u32,
        n_treasurer: u32,
        n_fields: u32,
    ) -> TestAssociationUsers {
        let admin_claim = Claims {
            sub: Some(self.admin.id),
            exp: 0,
            email: self.admin.email.clone(),
        };
        println!(
            "Creating association with {} members, {} treasurers and {} fields",
            n_member, n_treasurer, n_fields
        );
        let config = Config::new();

        let claims = Claims {
            sub: Some(self.admin.id),
            exp: 0,
            email: self.admin.email.clone(),
        };
        let schema = self.get_schema_for_tests(config.clone(), claims.clone());

        let all_users = n_member + n_treasurer;
        let users_json = create_users_json(all_users);
        let user_id_futures = users_json
            .iter()
            .map(async |user| {
                let request = async_graphql::Request::new(user.to_string());
                let response = schema.execute(request).await;
                if response.is_err() {
                    panic!("Error executing request: {:?}", response);
                }
                let response = &response
                    .data
                    .into_json()
                    .expect("Something went wrong parsing the response")["createUser"];

                let user: User =
                    serde_json::from_value(response.clone()).expect("Should deserialize user");
                user
            })
            .collect::<Vec<_>>();
        let users: Vec<User> = join_all(user_id_futures).await;

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
                id,
                name,
                neighborhood,
                country,
                state,
                address,
                identity,
                public,
                createdAt,
                updatedAt,
            }}
            }}"#,
        );
        let request = async_graphql::Request::new(create_association_mutation).data(claims);
        let response = schema
            .execute(request)
            .await
            .data
            .into_json()
            .expect("Something went wrong parsing the response");
        let association_json = response["createAssociation"]
            .as_object()
            .expect("Should get association object");
        let association_json =
            serde_json::to_string(association_json).expect("Should serialize association");
        let association = serde_json::from_str::<Association>(&association_json)
            .expect("Should deserialize association");

        let association_id = association.id;

        let user_ids = users.iter().map(|user| user.id).collect::<Vec<Uuid>>();
        let user_memberships_request = create_user_membership(user_ids.clone(), association_id);
        let create_treasurer_requests = create_treasurers(
            user_ids[..n_treasurer as usize].to_vec(),
            association_id,
            NaiveDate::from_ymd_opt(2020, 01, 01).unwrap(),
            NaiveDate::from_ymd_opt(2100, 01, 01).unwrap(),
        );

        for (idx, user_memberships_request) in user_memberships_request.iter().enumerate() {
            let user_claim = Claims {
                sub: Some(user_ids[idx]),
                exp: 0,
                email: Some(format!("test{}@gmail.com", idx)),
            };

            let request = async_graphql::Request::new(user_memberships_request).data(user_claim);

            let response = schema
                .execute(request)
                .await
                .data
                .into_json()
                .expect("Something went wrong parsing the response");
            let user_id = response["associate"]["userId"]
                .as_str()
                .expect("Should get id from user");
            Uuid::parse_str(user_id).expect("Should parse id to Uuid");

            if (idx as u32) < n_treasurer {
                let request = async_graphql::Request::new(create_treasurer_requests[idx].clone())
                    .data(admin_claim.clone());
                let response = schema
                    .execute(request)
                    .await
                    .data
                    .into_json()
                    .expect("Something went wrong parsing the response");
                let user_id = response["createAssociationTreasurer"]["userId"]
                    .as_str()
                    .expect("Should get id from user");
                Uuid::parse_str(user_id).expect("Should parse id to Uuid");
            }
        }

        let fields_query = create_fields(n_fields, association_id);

        let fields = fields_query
            .into_iter()
            .map(async |field_query| {
                let request = async_graphql::Request::new(field_query).data(admin_claim.clone());
                let response = schema
                    .execute(request)
                    .await
                    .data
                    .into_json()
                    .expect("Something went wrong parsing the response");
                let field_json = response["createField"]
                    .as_object()
                    .expect("Should get field object");
                let field_json = serde_json::to_string(field_json).expect("Should serialize field");
                let field: Field =
                    serde_json::from_str(&field_json).expect("Should deserialize field");
                field
            })
            .collect::<Vec<_>>();
        let fields = join_all(fields).await;

        let treasurers = users
            .clone()
            .iter()
            .take(n_treasurer as usize)
            .cloned()
            .collect::<Vec<_>>();
        TestAssociationUsers {
            association,
            members: users.clone(),
            treasurers,
            fields,
        }
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
                        id,
                        name,
                        birthday,
                        address,
                        activity,
                        email,
                        personalPhone,
                        commercialPhone,
                        usesWhatsapp,
                        identities,
                        profileUrl,
                        createdAt,
                        updatedAt
                    }}
                }}
                "#,
                id, id
            )
        })
        .collect::<Vec<String>>()
}

pub fn create_user_membership(user_ids: Vec<Uuid>, association_id: Uuid) -> Vec<String> {
    user_ids
        .into_iter()
        .map(|user_id| {
            format!(
                r#"mutation {{
                    associate(associationId: "{}")
                    {{
                        userId
                    }}
                }}"#,
                association_id
            )
        })
        .collect()
}

pub fn create_treasurers(
    user_ids: Vec<Uuid>,
    association_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Vec<String> {
    user_ids
        .into_iter()
        .map(|user_id| {
            format!(
                r#"mutation {{
                    createAssociationTreasurer(userIdTreasurer: "{}", associationId: "{}", startDate: "{}", endDate: "{}")
                    {{
                        userId
                    }}
                }}"#,
                user_id, association_id, start_date, end_date
            )
        })
        .collect()
}

pub fn create_fields(n_fields: u32, association_id: Uuid) -> Vec<String> {
    let json_rule = r#"{\"reservations_start_at_time_utc\":\"06:00:00\",\"max_duration_minutes\":60,\"max_reservations_per_period\":1,\"reservation_period\":\"Daily\"}"#;

    (0..n_fields)
        .into_iter()
        .map(|id| {
            format!(
                r#"mutation {{
                    createField(fieldInput: {{
                        associationId: "{}", name: "Test Field {}",
                        description: "Test field description",
                        reservationRules: "{}",
                        latitude: -16.42,
                        longitude: -39.07
                    }})
                    {{
                        id,
                        associationId,
                        name,
                        description,
                        reservationRules,
                        latitude,
                        longitude,
                        createdAt,
                        updatedAt
                    }}
                }}"#,
                association_id, id, json_rule
            )
        })
        .collect()
}

pub fn create_reservation(
    field_id: Uuid,
    user_id: Uuid,
    description: String,
    start_date: DateTime<chrono::Utc>,
    end_date: DateTime<chrono::Utc>,
) -> String {
    format!(
        r#"mutation {{
            createFieldReservation(fieldReservationInput: {{
                fieldId: "{}", userId: "{}",
                description: "{}",
                startDate: "{}",
                endDate: "{}"
            }})
            {{
                id,
                fieldId,
                userId,
                description,
                startDate,
                endDate,
                deleted,
                createdAt,
                updatedAt
            }}
        }}"#,
        field_id, user_id, description, start_date, end_date
    )
}

pub struct FixedClock(DateTime<Utc>);

#[async_trait]
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}
