use async_graphql::{Context, FieldResult, Object, SimpleObject};
use sqlx::QueryBuilder;
use uuid::Uuid;

use crate::{
    relations::model::{Relations, Role},
    token::Claims,
    DB,
};

use super::model::{Association, AssociationInput, AssociationUpdate};

#[derive(SimpleObject)]
pub struct AssociationsPage {
    pub total_size: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
    pub has_previous_page: bool,
    pub has_next_page: bool,
    pub items: Vec<Association>,
}

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

        // Normaliza page e page_size
        let page_size = if page_size > 0 { page_size } else { 100 };
        let mut page = if page > 0 { page } else { 1 };

        let member_only = member.unwrap_or(false);
        let search_pattern = search.map(|s| format!("%{}%", s));

        let mut count_qb = QueryBuilder::new(
            r#"SELECT COUNT(DISTINCT a.id) FROM "Association" a"#,
        );

        if member_only {
            count_qb
                .push(r#" JOIN "AssociationRoles" ar ON a.id = ar.association_id AND ar.user_id = "#)
                .push_bind(user_id.clone())
                .push(" WHERE ar.user_id IS NOT NULL");
        } else {
            count_qb.push(" WHERE a.public = TRUE");
        }

        if let Some(ref pat) = search_pattern {
            count_qb
                .push(" AND a.name ILIKE ")
                .push_bind(pat);
        }

        let total_size: i64 = count_qb
            .build_query_scalar()
            .fetch_one(pool)
            .await?;

        let total_pages = (total_size + page_size - 1) / page_size;
        page = page.min(total_pages.max(1));
        let offset = (page - 1) * page_size;

        let mut qb = QueryBuilder::new(
            r#"SELECT DISTINCT a.* FROM "Association" a"#,
        );

        if member_only {
            qb
                .push(" JOIN \"AssociationRoles\" ar ON a.id = ar.association_id AND ar.user_id = ")
                .push_bind(user_id)
                .push(" WHERE ar.user_id IS NOT NULL");
        } else {
            qb.push(" WHERE a.public = TRUE");
        }

        if let Some(ref pat) = search_pattern {
            qb
                .push(" AND a.name ILIKE ")
                .push_bind(pat);
        }
        
        qb
            .push(" ORDER BY a.name")
            .push(" LIMIT ")
            .push_bind(page_size)
            .push(" OFFSET ")
            .push_bind(offset);

        let items: Vec<Association> = qb
            .build_query_as()
            .fetch_all(pool)
            .await?;

        // Retorna o page object
        Ok(AssociationsPage {
            total_size,
            page,
            page_size,
            total_pages,
            has_previous_page: page > 1,
            has_next_page:     page < total_pages,
            items,
        })
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
