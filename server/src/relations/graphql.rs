use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{token::Claims, user::model::User, DB};

use super::model::{AssociationAdmin, AssociationTreasurer, Relations, UserAssociation};

#[derive(Default)]
pub struct RelationsMutation;

#[Object(extends)]
impl RelationsMutation {
    async fn associate(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> FieldResult<UserAssociation> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;
        let pool = ctx.data::<DB>().unwrap();
        let user = Relations::create_user_association(pool, user_id, association_id).await?;
        Ok(user)
    }

    async fn create_association_treasurer(
        &self,
        ctx: &Context<'_>,
        user_id_treasurer: Uuid,
        association_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
    ) -> FieldResult<AssociationTreasurer> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;
        let pool = ctx.data::<DB>().unwrap();

        let user = User::read_one(pool, &user_id).await?;
        if user.is_admin(ctx, association_id).await? || claims.is_global_admin() {
            Err(anyhow::Error::msg("Only user admin can set treasurer"))?
        }

        let association_treasurer = Relations::create_treasurer(
            pool,
            user_id_treasurer,
            association_id,
            start_date,
            end_date,
        )
        .await?;
        Ok(association_treasurer)
    }

    async fn create_association_admin(
        &self,
        ctx: &Context<'_>,
        user_id_admin: Uuid,
        association_id: Uuid,
    ) -> FieldResult<AssociationAdmin> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;
        let pool = ctx.data::<DB>().unwrap();

        let user = User::read_one(pool, &user_id).await?;
        if user.is_admin(ctx, association_id).await? || claims.is_global_admin() {
            Err(anyhow::Error::msg("Only user admin can set other admins"))?
        }

        let association_admin =
            Relations::create_admin(pool, user_id_admin, association_id).await?;
        Ok(association_admin)
    }
}
