use async_graphql::{InputObject, Object, SimpleObject, Context};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DB;

#[derive(FromRow, Deserialize, Serialize)]
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

#[derive(SimpleObject, sqlx::FromRow)]
pub struct TransactionCreator {
    pub id: Uuid,
    pub name: String,
}

#[Object]
impl Transaction{
    pub async fn id(&self) -> Uuid{
        self.id
    }
    pub async fn association_id(&self) -> Uuid{
        self.association_id
    }
    pub async fn creator_id(&self) -> Uuid{
        self.creator_id
    }
    pub async fn details(&self) -> String{
        self.details.clone()
    }
    pub async fn amount(&self) -> BigDecimal{
        self.amount.clone()
    }
    pub async fn proof_url(&self) -> Option<String>{
        self.proof_url.clone()
    }
    pub async fn reference_date(&self) -> chrono::NaiveDate{
        self.reference_date
    }
    pub async fn deleted(&self) -> bool{
        self.deleted
    }
    pub async fn created_at(&self) -> chrono::NaiveDateTime{
        self.created_at
    }
    pub async fn updated_at(&self) -> chrono::NaiveDateTime{
        self.updated_at
    }

    async fn creator<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> async_graphql::Result<Option<TransactionCreator>> {
        let pool = ctx.data::<DB>()?;

        let creator = sqlx::query_as!(
            TransactionCreator,
            r#"SELECT id, name 
            FROM "User" WHERE id = $1"#,
            self.creator_id
        )
        .fetch_optional(&*pool)
        .await
        .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(creator)
    }
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
