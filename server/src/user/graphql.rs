use async_graphql::{Context, FieldResult, InputObject, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::User;

#[derive(Default)]
pub struct AssociationQuery;

#[Object(extends)]
impl AssociationQuery {
    // Query user.
    async fn user(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = User::read_one(pool, &id).await?;
        Ok(user)
    }
}

#[derive(Default)]
pub struct AssociationMutation;

#[Object(extends)]
impl AssociationMutation {
    // Mutate user.
    async fn create_user(&self, ctx: &Context<'_>, user: UserInput) -> FieldResult<User> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = User::create(pool, user).await?;
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
