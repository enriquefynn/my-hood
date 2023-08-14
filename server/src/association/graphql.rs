use async_graphql::{Context, FieldResult, InputObject, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::Association;

#[derive(Default)]
pub struct UserQuery;

#[Object(extends)]
impl UserQuery {
    // Query association.
    async fn association(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<Association> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = Association::read_one(pool, &id).await?;
        Ok(user)
    }
}

#[derive(Default)]
pub struct UserMutation;

#[Object(extends)]
impl UserMutation {
    // Mutate association.
    async fn create_association(
        &self,
        ctx: &Context<'_>,
        association: AssociationInput,
    ) -> FieldResult<Association> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = Association::create(pool, association).await?;
        Ok(user)
    }
}

#[derive(InputObject)]
pub struct AssociationInput {
    pub name: String,
    pub neighborhood: String,
    pub country: String,
    pub state: String,
    pub address: String,
    pub identity: Option<String>,
}
