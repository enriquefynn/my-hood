use async_graphql::{Context, FieldResult, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Transaction, TransactionInput};

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
