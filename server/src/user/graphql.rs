use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{relations::model::Relations, token::Claims, DB};

use super::model::{User, UserInput};

#[derive(Default)]
pub struct UserQuery;

#[Object(extends)]
impl UserQuery {
    // Query user.
    async fn user(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<User> {
        let claims = ctx.data::<Claims>()?;
        let user_id = &claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        if &id == user_id || claims.is_global_admin() {
            let pool = ctx.data::<DB>().unwrap();
            let user = User::read_one(pool, &id).await?;
            Ok(user)
        } else {
            return Err(
                anyhow::Error::msg("User cannot know information about other users").into(),
            );
        }
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object(extends)]
impl UserMutation {
    // Mutate user.
    async fn create_own_user(&self, ctx: &Context<'_>, user_input: UserInput) -> FieldResult<User> {
        let claims = ctx.data::<Claims>()?;
        let user_email = &claims.email;
        if &user_input.email != user_email {
            return Err(anyhow::Error::msg("Unauthorized, please log in").into());
        }

        let pool = ctx.data::<DB>().expect("DB pool not found");
        let user = User::create(pool, user_input).await?;
        Ok(user)
    }

    async fn create_user(&self, ctx: &Context<'_>, user_input: UserInput) -> FieldResult<User> {
        let claims = ctx.data::<Claims>()?;
        if !claims.is_global_admin() {
            return Err(anyhow::Error::msg("Unauthorized, please log in").into());
        }

        let pool = ctx.data::<DB>().expect("DB pool not found");
        let user = User::create(pool, user_input).await?;
        Ok(user)
    }

    async fn toggle_pending_user(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> FieldResult<bool> {
        let claims = ctx.data::<Claims>()?;
        let user_id = &claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        let is_association_admin = Relations::is_admin(ctx, user_id, association_id).await?;
        if is_association_admin || claims.is_global_admin() {
            let pool = ctx.data::<DB>().expect("DB pool not found");
            let toggle_user = User::toggle_approve(pool, &user_id, &association_id).await?;
            Ok(toggle_user)
        } else {
            Err(anyhow::Error::msg("Unauthorized, please log in").into())
        }
    }
}
