use std::ops::Range;

use async_graphql::{Context, Enum, SimpleObject};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::DB;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum, sqlx::Type)]
#[sqlx(type_name = "association_role")]
#[sqlx(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Treasurer,
    Member,
}

#[derive(Debug, SimpleObject, FromRow)]
pub struct AssociationRoles {
    pub user_id: Uuid,
    pub association_id: Uuid,
    pub role: Role,
    pub pending: bool,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, SimpleObject, FromRow)]
pub struct AssociationRolesUpdate {
    pub user_id: Uuid,
    pub association_id: Uuid,
    pub role: Role,
    pub pending: bool,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
}

pub struct Relations;

impl Relations {
    pub async fn create_association_role<'e, E>(
        executor: E,
        user_id: Uuid,
        association_id: Uuid,
        role: Role,
        pending: bool,
        mandate: Option<Range<chrono::NaiveDate>>,
    ) -> Result<AssociationRoles, anyhow::Error>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        let start_date = mandate.clone().map(|m| m.start);
        let end_date = mandate.map(|m| m.end);
        let user_association = sqlx::query_as::<_, AssociationRoles>(
            r#"INSERT INTO "AssociationRoles" (user_id, association_id, role, pending, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *"#,
        )
        .bind(user_id)
        .bind(association_id)
        .bind(role)
        .bind(pending)
        .bind(start_date)
        .bind(end_date)
        .fetch_one(executor)
        .await?;

        Ok(user_association)
    }

    pub async fn get_role(
        ctx: &Context<'_>,
        user_id: &Uuid,
        association_id: Uuid,
        role: Role,
    ) -> Result<Option<AssociationRoles>, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let association_roles = sqlx::query_as::<_, AssociationRoles>(
            r#"SELECT * FROM "AssociationRoles" WHERE
            user_id = $1 AND 
            association_id = $2 AND 
            role = $3"#,
        )
        .bind(user_id)
        .bind(association_id)
        .bind(role)
        .fetch_optional(pool)
        .await?;

        Ok(association_roles)
    }

    pub async fn update_role(
        ctx: &Context<'_>,
        association_roles: AssociationRolesUpdate,
    ) -> Result<AssociationRoles, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let user_association = sqlx::query_as::<_, AssociationRoles>(
            r#"UPDATE "AssociationRoles" SET
                role = COALESCE($3, role),
                pending = COALESCE($4, pending),
                start_date = COALESCE($5, start_date),
                end_date = COALESCE($6, end_date)
                WHERE user_id = $1 AND association_id = $2
                RETURNING *
            "#,
        )
        .bind(association_roles.user_id)
        .bind(association_roles.association_id)
        .bind(association_roles.role)
        .bind(association_roles.pending)
        .bind(association_roles.start_date)
        .bind(association_roles.end_date)
        .fetch_one(pool)
        .await?;

        Ok(user_association)
    }
}
