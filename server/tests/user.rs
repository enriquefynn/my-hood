mod queries;
mod test_utils;

use chrono::TimeZone;
#[cfg(test)]
use my_hood_server::config::Config;
use my_hood_server::{token::Claims, user::model::User};
use queries::{create_users, get_user};

#[tokio::test]
async fn test_create_user() {
    let now = chrono::Utc
        .with_ymd_and_hms(2024, 01, 01, 07, 0, 0)
        .unwrap();
    let test_db = test_utils::TestDatabase::new(now).await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(test_db.admin.id.clone()),
        exp: 0,
        email: test_db.admin.email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), claims);

    let create_user_mutation = r#"mutation {
            createOwnUser(userInput: {
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

    let claim = Claims {
        sub: None,
        exp: 0,
        email: Some("test@gmail.com".to_owned()),
    };

    let request = async_graphql::Request::new(create_user_mutation.to_string()).data(claim);
    let response = schema.execute(request).await.data.into_value();

    let expected_response = serde_json::from_str(
        r#"{
                "createOwnUser": {
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
    let now = chrono::Utc
        .with_ymd_and_hms(2024, 01, 01, 07, 0, 0)
        .unwrap();
    let test_db = test_utils::TestDatabase::new(now).await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(test_db.admin.id),
        exp: 0,
        email: test_db.admin.email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), claims);

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
        test_db.admin.id
    );
    let request = async_graphql::Request::new(get_user_query);
    let response = schema.execute(request).await.data.into_value();

    let expected_response = serde_json::from_str(
        r#"{
                "user": {
                    "name": "Test User 1",
                    "email": "default_user@test.com",
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
    let now = chrono::Utc
        .with_ymd_and_hms(2024, 01, 01, 07, 0, 0)
        .unwrap();
    let test_db = test_utils::TestDatabase::new(now).await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(test_db.admin.id),
        exp: 0,
        email: test_db.admin.email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), claims);

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
    let now = chrono::Utc
        .with_ymd_and_hms(2024, 01, 01, 07, 0, 0)
        .unwrap();
    let test_db = test_utils::TestDatabase::new(now).await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(test_db.admin.id),
        exp: 0,
        email: test_db.admin.email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), claims);

    let users = create_users(10);
    for (i, create_user) in users.iter().enumerate() {
        let claim = Claims {
            sub: None,
            exp: 0,
            email: Some(format!("test{}@gmail.com", i)),
        };
        let request = async_graphql::Request::new(create_user.to_string()).data(claim);
        let response = &schema
            .execute(request)
            .await
            .data
            .into_json()
            .expect("Failed to convert response to JSON")["createOwnUser"];
        let user =
            serde_json::from_value::<User>(response.clone()).expect("Failed to deserialize user");
        let get_user_query = get_user(user.id);
        let claim = Claims {
            sub: Some(user.id),
            exp: 0,
            email: user.email.clone(),
        };
        let request = async_graphql::Request::new(get_user_query.to_string()).data(claim);
        let response = &schema
            .execute(request)
            .await
            .data
            .into_json()
            .expect("Failed to convert response to JSON")["user"];
        let get_user =
            serde_json::from_value::<User>(response.clone()).expect("Failed to deserialize user");

        assert_eq!(user, get_user);
    }
}
