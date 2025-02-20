use async_graphql::{InputObject, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DB;

#[derive(SimpleObject, FromRow, Deserialize, Serialize)]
pub struct Transaction {
    pub id: Uuid,
    pub association_id: Uuid,
    pub creator_id: Uuid,
    pub details: String,
    pub amount: BigDecimal,
    pub reference_date: chrono::NaiveDate,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(InputObject)]
pub struct TransactionInput {
    association_id: Uuid,
    creator_id: Uuid,
    details: String,
    amount: sqlx::types::BigDecimal,
    reference_date: chrono::NaiveDate,
}

impl Transaction {
    pub async fn create(
        db: &DB,
        transaction: TransactionInput,
    ) -> Result<Transaction, anyhow::Error> {
        let mut tx = db.begin().await?;

        let user = sqlx::query_as!(
            Transaction,
            r#"
            WITH valid_treasurer AS (
                SELECT 1
                FROM "AssociationTreasurer"
                WHERE association_id = $1 AND user_id = $2
            )
            INSERT INTO "Transaction" (association_id, creator_id, details, amount,
                reference_date)
                SELECT $1, $2, $3, $4, $5
                FROM valid_treasurer
                RETURNING *"#,
            transaction.association_id,
            transaction.creator_id,
            transaction.details,
            transaction.amount,
            transaction.reference_date,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(user)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<Transaction, anyhow::Error> {
        let mut tx = db.begin().await?;
        let user = sqlx::query_as!(
            Transaction,
            r#"SELECT * FROM "Transaction" WHERE id = $1"#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        Ok(user)
    }

    pub async fn read_all(db: DB) -> Result<Vec<Transaction>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let users = sqlx::query_as!(Transaction, r#"SELECT * FROM "Transaction""#)
            .fetch_all(&mut *tx)
            .await?;
        Ok(users)
    }
}
