use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DB;

use super::graphql::AssociationInput;

#[derive(SimpleObject, FromRow, Deserialize, Serialize)]
pub struct Association {
    id: Uuid,
    name: String,
    neighborhood: String,
    country: String,
    state: String,
    address: String,
    identity: Option<String>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

impl Association {
    pub async fn create(
        db: &DB,
        association: AssociationInput,
    ) -> Result<Association, anyhow::Error> {
        let mut tx = db.begin().await?;

        let user = sqlx::query_as!(
            Association,
            r#"INSERT INTO Association (name, neighborhood, country, state, address,
                identity)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *"#,
            association.name,
            association.neighborhood,
            association.country,
            association.state,
            association.address,
            association.identity,
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(user)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<Association, anyhow::Error> {
        let mut tx = db.begin().await?;
        let user = sqlx::query_as!(
            Association,
            r#"SELECT * FROM Association WHERE id = $1"#,
            id
        )
        .fetch_one(&mut tx)
        .await?;
        Ok(user)
    }

    pub async fn read_all(db: DB) -> Result<Vec<Association>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let users = sqlx::query_as!(Association, r#"SELECT * FROM Association"#)
            .fetch_all(&mut tx)
            .await?;
        Ok(users)
    }
}
