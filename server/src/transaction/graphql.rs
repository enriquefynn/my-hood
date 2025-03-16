use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{token::Claims, user::model::User, DB};

use super::model::{Transaction, TransactionInput};

#[derive(Default)]
pub struct TransactionQuery;

#[Object(extends)]
impl TransactionQuery {
    async fn transaction(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<Transaction> {
        let pool = ctx.data::<DB>().unwrap();
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
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        let pool = ctx.data::<DB>().unwrap();
        let user = User::read_one(pool, &user_id).await?;
        let is_treasuerer = user.is_treasurer(ctx, transaction.association_id).await?;
        if !is_treasuerer {
            Err(anyhow::Error::msg("Unauthorized, please log in"))?
        }
        let user = Transaction::create(pool, transaction).await?;
        Ok(user)
    }
}
