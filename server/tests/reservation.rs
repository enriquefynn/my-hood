mod test_utils;

use chrono::TimeZone;
#[cfg(test)]
use my_hood_server::config::Config;
use my_hood_server::{field::model::FieldReservation, token::Claims};
use test_utils::{create_reservation, TestDatabase};

#[tokio::test]
async fn test_create_reservation() {
    let now = chrono::Utc
        .with_ymd_and_hms(2024, 01, 01, 07, 0, 0)
        .unwrap();
    let test_db = TestDatabase::new(now).await;
    let config = Config::new();

    let test_data = test_db
        .create_association_admin_member_treasury_fields(10, 2, 1)
        .await;

    let field_id = test_data.fields[0].id;
    let user_id = test_data.members[0].id;
    let description = "Test reservation for beach tennis".to_owned();
    let start_date = "2023-10-01T10:00:00Z".to_string().parse().unwrap();
    let end_date = "2023-10-01T11:00:00Z".to_string().parse().unwrap();

    let user_0_claim = Claims {
        sub: Some(test_data.members[0].id),
        exp: 0,
        email: test_data.members[0].email.clone(),
    };
    let schema = test_db.get_schema_for_tests(config.clone(), user_0_claim);

    let reservation_query =
        create_reservation(field_id, user_id, description, start_date, end_date);

    let request = async_graphql::Request::new(reservation_query);
    let response = schema.execute(request).await;
    if response.is_err() {
        panic!("Error executing request: {:?}", response);
    }
    let response = &response
        .data
        .into_json()
        .expect("Failed to convert response to JSON")["createFieldReservation"];

    let reservation = serde_json::from_value::<FieldReservation>(response.clone())
        .expect("Failed to deserialize reservation");
    println!("Reservation: {:?}", reservation);
}
