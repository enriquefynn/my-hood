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

    /// Busca associações com filtros (search, member) e paginação offset-based.
    async fn associations_paginated(
        &self,
        ctx: &Context<'_>,
        search: Option<String>,
        member: Option<bool>,
        #[graphql(default = 1)]      page_arg: i64,
        #[graphql(name = "pageSize", default = 100)] page_size_arg: i64,
    ) -> FieldResult<AssociationsPage> {
        // 1) Autenticação e pool
        let claims = ctx.data::<Claims>()?;
        let user_id = claims
            .sub
            .ok_or_else(|| anyhow::Error::msg("Unauthorized, please log in"))?;
        let pool = ctx.data::<DB>()?;

        // 2) Normaliza page e page_size
        let page_size = if page_size_arg > 0 { page_size_arg } else { 100 };
        let mut page = if page_arg > 0 { page_arg } else { 1 };

        // 3) Prepara os filtros comuns
        let member_only = member.unwrap_or(false);
        let search_pattern = search.map(|s| format!("%{}%", s));

        // 4) Conta total de registros com os mesmos filtros
        let total_size: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) 
              FROM "Association" a
              LEFT JOIN "AssociationRoles" ar
                ON a.id = ar.association_id
               AND ar.user_id = $2
             WHERE (
                   ($1::boolean AND ar.user_id IS NOT NULL)
                OR (NOT $1::boolean AND a.public = TRUE)
             )
               AND ($3::text IS NULL OR a.name ILIKE $3)
            "#,
        )
        .bind(member_only)              // $1
        .bind(user_id)                  // $2
        .bind(search_pattern.clone())   // $3
        .fetch_one(pool)
        .await?;

        // 5) Calcula total_pages e ajusta page dentro dos limites
        let total_pages = (total_size + page_size - 1) / page_size;
        page = page.min(total_pages.max(1));

        // 6) Offset para a página atual
        let offset = (page - 1) * page_size;

        // 7) Busca os registros da página
        let items: Vec<Association> = sqlx::query_as(
            r#"
            SELECT a.* 
              FROM "Association" a
              LEFT JOIN "AssociationRoles" ar
                ON a.id = ar.association_id
               AND ar.user_id = $2
             WHERE (
                   ($1::boolean AND ar.user_id IS NOT NULL)
                OR (NOT $1::boolean AND a.public = TRUE)
             )
               AND ($3::text IS NULL OR a.name ILIKE $3)
             ORDER BY a.name
             LIMIT $4 OFFSET $5
            "#,
        )
        .bind(member_only)              // $1
        .bind(user_id)                  // $2
        .bind(search_pattern.clone())   // $3
        .bind(page_size)                // $4
        .bind(offset)                   // $5
        .fetch_all(pool)
        .await?;

        // 8) Monta o objeto de retorno
        let page_data = AssociationsPage {
            total_size,
            page,
            page_size,
            total_pages,
            has_previous_page: page > 1,
            has_next_page: page < total_pages,
            items,
        };

        Ok(page_data)
    }

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

        // let mut sql = String::from(
        //     r#"
        //     SELECT DISTINCT a.*
        //     FROM "Association" a
        //     "#,
        // );
        
        // if member_only {
        //     sql.push_str(
        //         r#"
        //         LEFT JOIN "AssociationRoles" ar 
        //             ON a.id = ar.association_id
        //             AND ar.user_id = $1
        //         WHERE ar.user_id IS NOT NULL
        //         "#,
        //     );
        // } else {
        //     sql.push_str(" WHERE a.public = TRUE");
        // }
        
        // if search_pattern.is_some() {
        //     let idx = if member_only { 2 } else { 1 };
        //     sql.push_str(&format!(" AND a.name ILIKE ${}", idx));
        // }
        
        // let mut query = sqlx::query_as::<_, Association>(&sql);

        // if member_only {
        //     query = query.bind(user_id);
        // }
        
        // if search_pattern.is_some() {
        //     query = query.bind(search_pattern);
        // }

        // let associations = query.fetch_all(pool).await?;

        // Ok(associations)
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
