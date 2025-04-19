use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{
    relations::model::{Relations, Role},
    token::Claims,
    DB,
};

use super::model::{Association, AssociationInput, AssociationUpdate};

#[derive(Default)]
pub struct AssociationQuery;

#[Object(extends)]
impl AssociationQuery {
    // Query association.
    async fn association(&self, ctx: &Context<'_>, id: Uuid) -> FieldResult<Association> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        let pool = ctx.data::<DB>().unwrap();
        let association = Association::read_one(pool, &id).await?;

        if association.public {
            Ok(association)
        } else {
            let member_role = Relations::get_role(ctx, &user_id, id, Role::Member).await?;
            if member_role.is_some() {
                Ok(association)
            } else {
                Err(anyhow::Error::msg(
                    "User is unauthorized to view association",
                ))?
            }
        }
    }
}

#[derive(Default)]
pub struct AssociationMutation;

#[Object]
impl AssociationMutation {
    async fn create_association(
        &self,
        ctx: &Context<'_>,
        association: AssociationInput,
    ) -> FieldResult<Association> {
        let claims = ctx.data::<Claims>()?;

        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        let pool = ctx.data::<DB>().unwrap();
        let association = Association::create(pool, user_id, association).await?;
        Ok(association)
    }

    async fn update_association(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
        association: AssociationUpdate,
    ) -> FieldResult<Association> {
        let claims = ctx.data::<Claims>()?;

        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        let pool = ctx.data::<DB>().unwrap();
        let is_admin = Relations::get_role(ctx, &user_id, association_id, Role::Admin).await?;
        if is_admin.is_none() {
            Err(anyhow::Error::msg(
                "User is unauthorized to update association",
            ))?
        }

        let association = Association::update(pool, &association_id, association).await?;
        Ok(association)
    }
}
