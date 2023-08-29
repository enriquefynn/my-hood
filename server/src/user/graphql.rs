use async_graphql::{Context, FieldResult, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{User, UserInput};

#[derive(Default)]
pub struct UserQuery;

#[Object(extends)]
impl UserQuery {
    // Query user.
    async fn user(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = User::read_one(pool, &id).await?;
        Ok(user)
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object(extends)]
impl UserMutation {
    // Mutate user.
    async fn create_user(&self, ctx: &Context<'_>, user: UserInput) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = User::create(pool, user).await?;
        Ok(user)
    }
}
