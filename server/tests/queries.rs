use chrono::{DateTime, NaiveDate};
use uuid::Uuid;

pub fn create_users(n_users: u32) -> Vec<String> {
    (0..n_users)
        .into_iter()
        .map(|id| {
            format!(
                r#"mutation {{
                    createOwnUser(userInput: {{
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

pub fn get_user(user_id: Uuid) -> String {
    format!(
        r#"query {{
                    user(id: "{}")
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
        user_id
    )
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

pub fn create_association(
    name: String,
    neighborhood: String,
    country: String,
    state: String,
    address: String,
) -> String {
    let create_association_mutation = format!(
        r#"mutation {{
        createAssociation(association: {{
            name: "{}",
            neighborhood: "{}",
            country: "{}",
            state: "{}",
            address: "{}",
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
        name, neighborhood, country, state, address
    );
    create_association_mutation
}
