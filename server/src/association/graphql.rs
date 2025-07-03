use async_graphql::{Context, FieldResult, Object};
use uuid::Uuid;

use crate::{
    association::model::{AssocFilter, AssociationsPage}, relations::model::{Relations, Role}, token::Claims, DB
};

use super::model::{Association, AssociationInput, AssociationUpdate};

#[derive(Default)]
pub struct AssociationQuery;

#[Object(extends)]
impl AssociationQuery {

    /// Searches for associations with optional filters for text and member status.
    ///
    /// - `search`: if provided, filters `a.name ILIKE %search%`
    /// - `member`: if `true`, only associations where the user (from JWT) is a member;
    ///             otherwise (default) only `a.public = true`.
    async fn associations(
        &self,
        ctx: &Context<'_>,
        search: Option<String>,
        member: Option<bool>,
        #[graphql(default = 1)] page: i64,
        #[graphql(name = "pageSize", default = 100)] page_size: i64,
    ) -> FieldResult<AssociationsPage> {
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or_else(|| anyhow::Error::msg("Unauthorized, please log in"))?;
        let pool = ctx.data::<DB>()?;

        let filter = AssocFilter {
            search,
            member_only: member.unwrap_or(false),
            user_id: if member.unwrap_or(false) { Some(user_id) } else { None },
            page,
            page_size,
        };

        let page_obj = Association::read_filtered_paginated(pool, filter).await?;
        Ok(page_obj)
    }
    
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
