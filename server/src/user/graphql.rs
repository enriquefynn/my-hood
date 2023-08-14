use async_graphql::{Context, EmptySubscription, FieldResult, InputObject, Object, Schema};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::User;

pub struct Query;

pub type ProjectSchema = Schema<Query, Mutation, EmptySubscription>;

#[Object(extends)]
impl Query {
    // Query user.
    async fn user(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = User::read_one(pool, &id).await?;
        Ok(user)
    }
}

pub struct Mutation;

#[Object(extends)]
impl Mutation {
    // Mutate user.
    async fn create_user(&self, ctx: &Context<'_>, input_user: UserInput) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = User::create(pool, input_user).await?;
        Ok(user)
    }
}

#[derive(InputObject)]
pub struct UserInput {
    pub name: String,
    pub birthday: chrono::NaiveDateTime,
    pub address: String,
    pub activity: Option<String>,
    pub email: Option<String>,
    pub personal_phone: Option<String>,
    pub commercial_phone: Option<String>,
    pub uses_whatsapp: bool,
    pub signed_at: chrono::NaiveDateTime,
    pub identities: String,
}
