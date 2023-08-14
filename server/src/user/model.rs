use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::{graphql::UserInput, DB};

#[derive(SimpleObject, FromRow, Deserialize, Serialize)]
pub struct User {
    id: Uuid,
    name: String,
    birthday: chrono::NaiveDateTime,
    address: String,
    activity: Option<String>,
    email: Option<String>,
    personal_phone: Option<String>,
    commercial_phone: Option<String>,
    uses_whatsapp: bool,
    signed_at: chrono::NaiveDateTime,
    identities: String,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

impl User {
    pub async fn create(db: &DB, user: UserInput) -> Result<User, anyhow::Error> {
        let mut tx = db.begin().await?;

        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO "User" (name, birthday, address, activity, email, personal_phone,
                commercial_phone, uses_whatsapp, signed_at, identities)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *"#,
            user.name,
            user.birthday,
            user.address,
            user.activity,
            user.email,
            user.personal_phone,
            user.commercial_phone,
            user.uses_whatsapp,
            user.signed_at,
            user.identities,
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(user)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<User, anyhow::Error> {
        let mut tx = db.begin().await?;
        let user = sqlx::query_as!(User, r#"SELECT * FROM "User" WHERE id = $1"#, id)
            .fetch_one(&mut tx)
            .await?;
        Ok(user)
    }

    pub async fn read_all(db: DB) -> Result<Vec<User>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let users = sqlx::query_as!(User, r#"SELECT * FROM "User""#)
            .fetch_all(&mut tx)
            .await?;
        Ok(users)
    }
}
