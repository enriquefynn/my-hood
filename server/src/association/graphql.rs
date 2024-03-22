use async_graphql::{Context, FieldResult, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{Association, AssociationInput};

#[derive(Default)]
pub struct AssociationQuery;

#[Object(extends)]
impl AssociationQuery {
    // Query association.
    async fn association(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<Association> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = Association::read_one(pool, &id).await?;
        Ok(user)
    }
}

#[derive(Default)]
pub struct AssociationMutation;

#[Object]
impl AssociationMutation {
    // Mutate association.
    async fn create_association(
        &self,
        ctx: &Context<'_>,
        association: AssociationInput,
    ) -> FieldResult<Association> {
        let pool = ctx.data::<PgPool>().unwrap();
        let association = Association::create(pool, association).await?;
        Ok(association)
    }
}
