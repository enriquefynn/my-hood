use async_graphql::{Context, FieldResult, InputObject, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::Transaction;

#[derive(Default)]
pub struct TransactionQuery;

#[Object(extends)]
impl TransactionQuery {
    async fn transaction(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<Transaction> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = Transaction::read_one(pool, &id).await?;
        Ok(user)
    }
}

#[derive(Default)]
pub struct TransactionMutation;

#[Object(extends)]
impl TransactionMutation {
    // Mutate association.
    async fn create_transaction(
        &self,
        ctx: &Context<'_>,
        transaction: TransactionInput,
    ) -> FieldResult<Transaction> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = Transaction::create(pool, transaction).await?;
        Ok(user)
    }
}

#[derive(InputObject)]
pub struct TransactionInput {
    pub association_id: Uuid,
    pub creator_id: Uuid,
    pub details: String,
    pub amount: sqlx::types::BigDecimal,
    pub reference_date: chrono::NaiveDate,
}
