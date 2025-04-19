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
    pub proof_url: Option<String>,
    pub reference_date: chrono::NaiveDate,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(InputObject)]
pub struct TransactionInput {
    pub association_id: Uuid,
    creator_id: Uuid,
    details: String,
    amount: sqlx::types::BigDecimal,
    reference_date: chrono::NaiveDate,
}

impl Transaction {
    pub async fn create(
        db: &DB,
        transaction_input: TransactionInput,
    ) -> Result<Transaction, anyhow::Error> {
        let mut tx = db.begin().await?;

        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            WITH valid_treasurer AS (
                SELECT 1
                FROM "AssociationRoles"
                WHERE association_id = $1 AND user_id = $2 AND role = 'treasurer'
            )
            INSERT INTO "Transaction" (association_id, creator_id, details, amount, reference_date)
                SELECT $1, $2, $3, $4, $5
                FROM valid_treasurer
                RETURNING *"#,
            transaction_input.association_id,
            transaction_input.creator_id,
            transaction_input.details,
            transaction_input.amount,
            transaction_input.reference_date,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(transaction)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<Transaction, anyhow::Error> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"SELECT * FROM "Transaction" WHERE id = $1"#,
            id
        )
        .fetch_one(&*db)
        .await?;
        Ok(transaction)
    }

    pub async fn read_all(db: &DB) -> Result<Vec<Transaction>, anyhow::Error> {
        let transactions = sqlx::query_as!(Transaction, r#"SELECT * FROM "Transaction""#)
            .fetch_all(&*db)
            .await?;
        Ok(transactions)
    }

    pub async fn toggle_deleted(db: &DB, id: &Uuid) -> Result<bool, anyhow::Error> {
        let mut tx = db.begin().await?;
        let deleted: bool = sqlx::query_scalar!(
            r#"
            UPDATE "Transaction"
            SET deleted = not deleted
            WHERE id = $1
            RETURNING deleted
            "#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(deleted)
    }
}

#[derive(SimpleObject, FromRow, Deserialize, Serialize)]
pub struct Charge {
    pub id: Uuid,
    pub association_id: Uuid,
    pub creator_id: Uuid,
    pub details: String,
    pub amount: BigDecimal,
    pub file_url: Option<String>,
    pub reference_date: chrono::NaiveDate,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(InputObject)]
pub struct ChargeInput {
    pub association_id: Uuid,
    creator_id: Uuid,
    details: Option<String>,
    amount: sqlx::types::BigDecimal,
    file_url: Option<String>,
    reference_date: chrono::NaiveDate,
}

impl Charge {
    // pub async fn create(db: &DB, charge_input: ChargeInput) -> Result<Charge, anyhow::Error> {
    //     let mut tx = db.begin().await?;

    //     let charge = sqlx::query_as!(
    //         Transaction,
    //         r#"
    //         WITH valid_treasurer AS (
    //             SELECT 1
    //             FROM "AssociationRoles"
    //             WHERE association_id = $1 AND user_id = $2 AND role = 'treasurer'
    //         )
    //         INSERT INTO "Charge" (association_id, creator_id, details, amount, reference_date)
    //             RETURNING *"#,
    //         charge_input.association_id,
    //         charge_input.creator_id,
    //         charge_input.details,
    //         charge_input.amount,
    //         charge_input.reference_date,
    //     )
    //     .fetch_one(&mut *tx)
    //     .await?;
    //     tx.commit().await?;
    //     Ok(transaction)
    // }
}
