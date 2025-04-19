use std::sync::Arc;

use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{
    relations::model::{Relations, Role},
    token::Claims,
    Clock, DB,
};

use super::model::{Field, FieldInput, FieldReservation, FieldReservationInput};

#[derive(Default)]
pub struct FieldQuery;
#[derive(Default)]
pub struct FieldMutation;

#[Object(extends)]
impl FieldQuery {
    async fn field(&self, _ctx: &Context<'_>, _id: Uuid) -> FieldResult<u32> {
        todo!()
    }
}

#[Object(extends)]
impl FieldMutation {
    async fn create_field(&self, ctx: &Context<'_>, field_input: FieldInput) -> FieldResult<Field> {
        let claims = ctx.data::<Claims>()?;
        let is_admin = Relations::get_role(
            ctx,
            &claims
                .sub
                .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?,
            field_input.association_id,
            Role::Admin,
        )
        .await?;
        if is_admin.is_none() {
            return Err(anyhow::Error::msg("User is not an admin of the association").into());
        }

        let pool = ctx.data::<DB>().expect("DB pool not found");
        let field = Field::create(pool, field_input).await?;
        Ok(field)
    }

    async fn create_field_reservation(
        &self,
        ctx: &Context<'_>,
        field_reservation_input: FieldReservationInput,
    ) -> FieldResult<FieldReservation> {
        let clock = ctx.data::<Arc<dyn Clock>>()?;
        let now: chrono::DateTime<chrono::Utc> = clock.now();

        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        if field_reservation_input.user_id != user_id {
            return Err(anyhow::Error::msg(
                "User id input does not match field reservation input id",
            )
            .into());
        }

        let pool = ctx.data::<DB>().expect("DB pool not found");
        let field = Field::get(pool, &field_reservation_input.field_id).await?;
        let member = Relations::get_role(ctx, &user_id, field.association_id, Role::Member).await?;
        if member.is_none() {
            return Err(anyhow::Error::msg("User is not a member of the association").into());
        }

        let field_reservation =
            FieldReservation::create(pool, &user_id, &field, field_reservation_input, now).await?;
        Ok(field_reservation)
    }

    async fn delete_field_reservation(
        &self,
        ctx: &Context<'_>,
        field_reservation_id: Uuid,
    ) -> FieldResult<FieldReservation> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or(anyhow::Error::msg("Unauthorized, please log in"))?;

        let pool = ctx.data::<DB>().expect("DB pool not found");
        let field_reservation = FieldReservation::get(pool, &field_reservation_id).await?;

        if field_reservation.user_id != user_id {
            return Err(
                anyhow::Error::msg("User id does not match field reservation input id").into(),
            );
        }
        let field = Field::get(pool, &field_reservation.field_id).await?;
        let member = Relations::get_role(ctx, &user_id, field.association_id, Role::Member).await?;
        if member.is_none() {
            return Err(anyhow::Error::msg("User is not a member of the association").into());
        }

        let field_reservation = FieldReservation::delete(pool, &field_reservation_id).await?;
        Ok(field_reservation)
    }
}
