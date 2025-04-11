mod test_utils;

#[cfg(test)]
use my_hood_server::config::Config;
use my_hood_server::token::Claims;

#[tokio::test]
async fn test_create_reservation() {
    let test_db = test_utils::TestDatabase::new().await;
    let config = Config::new();

    let claims = Claims {
        sub: Some(test_db.admin.id),
        exp: 0,
        email: test_db.admin.email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), claims);

    test_db.create_logins(10).await;
    test_db
        .create_association_admin_member_treasury_fields(10, 2, 1)
        .await;
}
