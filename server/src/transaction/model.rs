use async_graphql::SimpleObject;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DB;

use super::graphql::TransactionInput;

#[derive(SimpleObject, FromRow, Deserialize, Serialize)]
pub struct Transaction {
    id: Uuid,
    association_id: Uuid,
    creator_id: Uuid,
    details: String,
    amount: BigDecimal,
    reference_date: chrono::NaiveDate,
    deleted: bool,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

impl Transaction {
    pub async fn create(
        db: &DB,
        transaction: TransactionInput,
    ) -> Result<Transaction, anyhow::Error> {
        let mut tx = db.begin().await?;

        let user = sqlx::query_as!(
            Transaction,
            r#"INSERT INTO Transaction (association_id, creator_id, details, amount,
                reference_date)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING *"#,
            transaction.association_id,
            transaction.creator_id,
            transaction.details,
            transaction.amount,
            transaction.reference_date,
        )
        .fetch_one(&mut tx)
        .await?;
        tx.commit().await?;

        Ok(user)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<Transaction, anyhow::Error> {
        let mut tx = db.begin().await?;
        let user = sqlx::query_as!(
            Transaction,
            r#"SELECT * FROM Transaction WHERE id = $1"#,
            id
        )
        .fetch_one(&mut tx)
        .await?;
        Ok(user)
    }

    pub async fn read_all(db: DB) -> Result<Vec<Transaction>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let users = sqlx::query_as!(Transaction, r#"SELECT * FROM Transaction"#)
            .fetch_all(&mut tx)
            .await?;
        Ok(users)
    }
}
