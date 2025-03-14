#[cfg(test)]
use my_hood_server::config::Config;
use my_hood_server::token::{Claims, Roles};

use crate::{get_schema_for_tests, get_users_json, setup_db};

#[tokio::test]
async fn test_create_user() {
    let (db, admin) = setup_db().await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(admin.id),
        exp: 0,
        email: admin.email,
        roles: vec![Roles::GlobalAdmin],
    };
    let schema = get_schema_for_tests(db.clone(), config.clone(), claims);

    let create_user_mutation = r#"mutation {
            createUser(userInput: {
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
    let (db, admin) = setup_db().await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(admin.id),
        exp: 0,
        email: admin.email,
        roles: vec![Roles::GlobalAdmin],
    };
    let schema = get_schema_for_tests(db.clone(), config.clone(), claims);

    let create_user_mutation = r#"mutation {
            createUser(userInput: {
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
    let response = schema.execute(request).await;
    let response = response.data.into_value();
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
    let (db, admin) = setup_db().await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(admin.id),
        exp: 0,
        email: admin.email,
        roles: vec![Roles::GlobalAdmin],
    };
    let schema = get_schema_for_tests(db.clone(), config.clone(), claims);

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
    let (db, admin) = setup_db().await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(admin.id),
        exp: 0,
        email: admin.email,
        roles: vec![Roles::GlobalAdmin],
    };
    let schema = get_schema_for_tests(db.clone(), config.clone(), claims);

    let users = get_users_json(100);
    for create_user in users {
        let request = async_graphql::Request::new(create_user.to_string());
        let _response = schema.execute(request).await.data.into_value();
    }
}
