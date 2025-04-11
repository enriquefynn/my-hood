mod test_utils;

#[cfg(test)]
use my_hood_server::config::Config;
use my_hood_server::token::Claims;
use test_utils::create_users_json;

#[tokio::test]
async fn test_create_user() {
    let test_db = test_utils::TestDatabase::new().await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(test_db.admin.id.clone()),
        exp: 0,
        email: test_db.admin.email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), claims);

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
    let test_db = test_utils::TestDatabase::new().await;
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
    let test_db = test_utils::TestDatabase::new().await;
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
    let test_db = test_utils::TestDatabase::new().await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(test_db.admin.id),
        exp: 0,
        email: test_db.admin.email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), claims);

    let users = create_users_json(100);
    for create_user in users {
        let request = async_graphql::Request::new(create_user.to_string());
        let _response = schema.execute(request).await.data.into_value();
    }
}
