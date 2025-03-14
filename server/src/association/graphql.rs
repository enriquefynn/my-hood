use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{relations::model::Relations, token::Claims, DB};

use super::model::{Association, AssociationInput};

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
            let is_member = Relations::is_member(ctx, user_id, id).await?;
            if is_member {
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
    // Mutate association.
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
}
