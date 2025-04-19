use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{token::Claims, user::model::User, DB};

use super::model::{AssociationRoles, Relations, Role};

#[derive(Default)]
pub struct RelationsMutation;

#[Object(extends)]
impl RelationsMutation {
    async fn associate(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> FieldResult<AssociationRoles> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        let pool = ctx.data::<DB>().unwrap();
        let mut tx = pool.begin().await?;
        let user_role = Relations::create_association_role(
            &mut *tx,
            user_id,
            association_id,
            Role::Member,
            true,
            None,
        )
        .await?;
        tx.commit().await?;
        Ok(user_role)
    }

    async fn create_association_treasurer(
        &self,
        ctx: &Context<'_>,
        user_id_treasurer: Uuid,
        association_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> FieldResult<AssociationRoles> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;
        let pool = ctx.data::<DB>().unwrap();
        let mut tx = pool.begin().await?;

        let user = User::read_one(pool, &user_id).await?;
        if !user.is_admin(ctx, association_id).await? {
            Err(anyhow::Error::msg("Only user admin can set treasurer"))?
        }

        let association_treasurer = Relations::create_association_role(
            &mut *tx,
            user_id_treasurer,
            association_id,
            Role::Treasurer,
            false,
            Some(start_date..end_date),
        )
        .await?;
        tx.commit().await?;
        Ok(association_treasurer)
    }

    async fn create_association_admin(
        &self,
        ctx: &Context<'_>,
        user_id_admin: Uuid,
        association_id: Uuid,
    ) -> FieldResult<AssociationRoles> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;
        let pool = ctx.data::<DB>().unwrap();
        let mut tx = pool.begin().await?;

        let user = User::read_one(pool, &user_id).await?;
        if user.is_admin(ctx, association_id).await? {
            Err(anyhow::Error::msg("Only user admin can set other admins"))?
        }

        let association_admin = Relations::create_association_role(
            &mut *tx,
            user_id_admin,
            association_id,
            Role::Admin,
            false,
            None,
        )
        .await?;
        tx.commit().await?;
        Ok(association_admin)
    }
}
