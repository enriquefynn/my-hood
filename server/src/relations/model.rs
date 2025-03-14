use async_graphql::{Context, SimpleObject};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DB;

#[derive(SimpleObject, FromRow, Deserialize, Serialize)]
pub struct UserAssociation {
    user_id: Uuid,
    association_id: Uuid,
    pending: bool,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, SimpleObject)]
pub struct AssociationAdmin {
    pub user_id: Uuid,
    pub association_id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(SimpleObject)]
pub struct AssociationTreasurer {
    user_id: Uuid,
    association_id: Uuid,
    start_date: chrono::NaiveDate,
    end_date: Option<chrono::NaiveDate>,
    created_at: chrono::NaiveDateTime,
    updated_at: chrono::NaiveDateTime,
}

pub struct Relations;

impl Relations {
    pub async fn create_user_association(
        db: &DB,
        user_id: Uuid,
        association_id: Uuid,
    ) -> Result<UserAssociation, anyhow::Error> {
        let mut tx = db.begin().await?;

        let user_association = sqlx::query_as!(
            UserAssociation,
            r#"INSERT INTO "UserAssociation" (user_id, association_id)
                VALUES ($1, $2)
                RETURNING *"#,
            user_id,
            association_id,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(user_association)
    }

    pub async fn create_treasurer(
        db: &DB,
        user_id: Uuid,
        association_id: Uuid,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
    ) -> Result<AssociationTreasurer, anyhow::Error> {
        let mut tx = db.begin().await?;

        let association_treasurer = sqlx::query_as!(
            AssociationTreasurer,
            r#"INSERT INTO "AssociationTreasurer" (user_id, association_id, start_date, end_date)
                VALUES ($1, $2, $3, $4)
                RETURNING *"#,
            user_id,
            association_id,
            start_date,
            end_date
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(association_treasurer)
    }

    pub async fn create_admin(
        db: &DB,
        user_id: Uuid,
        association_id: Uuid,
    ) -> Result<AssociationAdmin, anyhow::Error> {
        let mut tx = db.begin().await?;

        let association_admin = sqlx::query_as!(
            AssociationAdmin,
            r#"INSERT INTO "AssociationAdmin" (user_id, association_id)
                VALUES ($1, $2)
                RETURNING *"#,
            user_id,
            association_id,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(association_admin)
    }

    pub async fn is_admin(
        ctx: &Context<'_>,
        user_id: &Uuid,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let is_admin = sqlx::query!(
            r#"SELECT user_id FROM "AssociationAdmin" WHERE user_id = $1 AND association_id = $2"#,
            user_id,
            association_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(is_admin.is_some())
    }

    pub async fn is_treasurer(
        ctx: &Context<'_>,
        user_id: Uuid,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let is_admin =sqlx::query!(
            r#"SELECT user_id FROM "AssociationTreasurer" WHERE user_id = $1 AND association_id = $2"#,
            user_id,
            association_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(is_admin.is_some())
    }

    pub async fn is_member(
        ctx: &Context<'_>,
        user_id: Uuid,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let is_member = sqlx::query!(
            r#"SELECT user_id FROM "UserAssociation" WHERE user_id = $1 AND association_id = $2"#,
            user_id,
            association_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(is_member.is_some())
    }

    pub async fn is_pending(
        ctx: &Context<'_>,
        user_id: Uuid,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let pending: bool = sqlx::query_scalar!(
            r#"SELECT pending FROM "UserAssociation" WHERE user_id = $1 AND association_id = $2"#,
            user_id,
            association_id
        )
        .fetch_one(pool)
        .await?;
        Ok(pending)
    }
}
