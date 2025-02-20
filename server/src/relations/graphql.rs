use async_graphql::{Context, FieldResult, Object};
use sqlx::PgPool;
use uuid::Uuid;

use super::model::{AssociationAdmin, AssociationTreasurer, Relations, UserAssociation};

#[derive(Default)]
pub struct RelationsMutation;

#[Object(extends)]
impl RelationsMutation {
    async fn create_user_association(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
        association_id: Uuid,
    ) -> FieldResult<UserAssociation> {
        let pool = ctx.data::<PgPool>().unwrap();
        let user = Relations::create_user_association(pool, user_id, association_id).await?;
        Ok(user)
    }

    async fn create_association_treasurer(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
        association_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
    ) -> FieldResult<AssociationTreasurer> {
        let pool = ctx.data::<PgPool>().unwrap();
        let association_treasurer =
            Relations::create_treasurer(pool, user_id, association_id, start_date, end_date)
                .await?;
        Ok(association_treasurer)
    }

    async fn create_association_admin(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
        association_id: Uuid,
    ) -> FieldResult<AssociationAdmin> {
        let pool = ctx.data::<PgPool>().unwrap();
        let association_admin = Relations::create_admin(pool, user_id, association_id).await?;
        Ok(association_admin)
    }
}
