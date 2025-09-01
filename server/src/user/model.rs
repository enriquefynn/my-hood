use async_graphql::{Context, InputObject, Object, SimpleObject};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use regex::Regex;

use crate::{
    association::model::Association,
    relations::model::{Relations, Role},
    DB,
};

#[derive(Debug, FromRow, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub password_hash: Option<String>,
    pub birthday: chrono::NaiveDate,
    pub address: String,
    pub activity: Option<String>,
    pub email: Option<String>,
    pub personal_phone: Option<String>,
    pub commercial_phone: Option<String>,
    pub uses_whatsapp: bool,
    pub identities: Option<String>,
    pub profile_url: Option<String>,
    pub deleted: Option<bool>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, InputObject, Deserialize)]
pub struct UserInput {
    // Set as pub so we can reuse this struct for Oauth.
    pub password_hash: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub profile_url: Option<String>,
    pub birthday: chrono::NaiveDate,
    pub address: String,
    pub activity: Option<String>,
    pub personal_phone: Option<String>,
    pub commercial_phone: Option<String>,
    pub uses_whatsapp: bool,
    pub identities: Option<String>,
}

#[derive(InputObject)]
pub struct UserUpdate {
    pub id: Uuid,
    pub name: Option<String>,
    pub birthday: chrono::NaiveDate,
    pub address: String,
    pub activity: Option<String>,
    pub personal_phone: Option<String>,
    pub commercial_phone: Option<String>,
    pub uses_whatsapp: bool,
    pub identities: Option<String>,
    pub profile_url: Option<String>,
    pub deleted: Option<bool>,
}

#[derive(SimpleObject, sqlx::FromRow)]
pub struct PendingMember{
    pub id: Uuid,
    pub name: String,
    pub birthday: chrono::NaiveDate,
    pub address: String,
    pub activity: Option<String>,
    pub email: Option<String>,
    pub personal_phone: Option<String>,
    pub commercial_phone: Option<String>,
    pub uses_whatsapp: bool,
    pub identities: Option<String>,
    pub profile_url: Option<String>,
    pub association_id: uuid::Uuid,
    pub association_name: String,
    pub requested_at: chrono::NaiveDateTime,
}

#[derive(async_graphql::SimpleObject)]
pub struct PendingMembersPage {
    pub total_size: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
    pub has_previous_page: bool,
    pub has_next_page: bool,
    pub items: Vec<PendingMember>,
}

static BCRYPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\$2[abyx]\$(\d{2})\$[./A-Za-z0-9]{53}$"#).unwrap()
});

#[Object]
impl User {
    pub async fn id(&self) -> Uuid {
        self.id
    }

    pub async fn name(&self) -> String {
        self.name.to_owned()
    }

    pub async fn birthday(&self) -> chrono::NaiveDate {
        self.birthday.to_owned()
    }

    pub async fn address(&self) -> String {
        self.address.to_owned()
    }

    pub async fn activity(&self) -> Option<String> {
        self.activity.to_owned()
    }

    pub async fn email(&self) -> Option<String> {
        self.email.to_owned()
    }

    pub async fn personal_phone(&self) -> Option<String> {
        self.personal_phone.to_owned()
    }

    pub async fn commercial_phone(&self) -> Option<String> {
        self.commercial_phone.to_owned()
    }

    pub async fn uses_whatsapp(&self) -> bool {
        self.uses_whatsapp
    }

    pub async fn identities(&self) -> Option<String> {
        self.identities.to_owned()
    }

    pub async fn profile_url(&self) -> Option<String> {
        self.profile_url.to_owned()
    }

    pub async fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    pub async fn updated_at(&self) -> chrono::NaiveDateTime {
        self.updated_at
    }

    pub async fn associations(&self, ctx: &Context<'_>) -> Result<Vec<Association>, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let associations = sqlx::query_as!(
            Association,
            r#"SELECT a.* FROM "Association" a
        INNER JOIN "AssociationRoles" ar ON a.id = ar.association_id WHERE ar.user_id = $1 AND ar.role = 'member'"#,
            self.id
        )
        .fetch_all(&*pool)
        .await?;
        Ok(associations)
    }

    pub async fn is_admin(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let role = Relations::get_role(ctx, &self.id, association_id, Role::Admin).await?;
        Ok(role.is_some())
    }

    pub async fn is_treasurer(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let role = Relations::get_role(ctx, &self.id, association_id, Role::Treasurer).await?;
        Ok(role.is_some())
    }

    pub async fn pending(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        let member = Relations::get_role(ctx, &self.id, association_id, Role::Member).await?;
        let pending = member.map(|m| m.pending).unwrap_or(false);
        Ok(pending)
    }
}

impl User {
    pub async fn create(db: &DB, mut user: UserInput) -> Result<User, anyhow::Error> {

        if let Some(plain) = user.password_hash.take() {
            let s: &str = plain.as_ref();

            if !(s.len() == 60 && BCRYPT_RE.is_match(s)) {
                let hashed = bcrypt::hash(plain, 12)?;
                user.password_hash = Some(hashed);
            }
        }

        let mut tx = db.begin().await?;

        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO "User" (password_hash, name, birthday, address, activity, email, personal_phone,
                commercial_phone, uses_whatsapp, identities, profile_url)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING *"#,
            user.password_hash,
            user.name,
            user.birthday,
            user.address,
            user.activity,
            user.email,
            user.personal_phone,
            user.commercial_phone,
            user.uses_whatsapp,
            user.identities,
            user.profile_url,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(user)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<User, anyhow::Error> {
        let user = sqlx::query_as!(User, r#"SELECT * FROM "User" WHERE id = $1"#, id)
            .fetch_one(&*db)
            .await?;
        Ok(user)
    }

    pub async fn read_one_by_email(db: &DB, email: &str) -> Result<Option<User>, anyhow::Error> {
        let user = sqlx::query_as!(User, r#"SELECT * FROM "User" WHERE email = $1"#, email)
            .fetch_optional(&*db)
            .await?;
        Ok(user)
    }

    pub async fn read_all(db: &DB) -> Result<Vec<User>, anyhow::Error> {
        let users = sqlx::query_as!(User, r#"SELECT * FROM "User""#)
            .fetch_all(&*db)
            .await?;
        Ok(users)
    }

    /// Return a paginated page of **association pending requests** (user requesting to join as `member`) 
    /// **only** in associations where `admin_user_id` is **admin (not pending)**.
    pub async fn read_pendings_paginated(db: &DB, admin_user_id: Uuid, mut page: i64, page_size: i64
    ) -> Result<PendingMembersPage, anyhow::Error> {
        // -------- COUNT ----------
        let total_size: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT (u.id, ar.association_id))
            FROM "AssociationRoles" ar
            JOIN "User" u ON u.id = ar.user_id
            JOIN "Association" a ON a.id = ar.association_id
            WHERE COALESCE(u.deleted, FALSE) = FALSE
            AND COALESCE(a.deleted, FALSE) = FALSE
            AND ar.role = 'member'
            AND ar.pending = TRUE
            AND u.id <> $1
            AND EXISTS (
                SELECT 1 FROM "AssociationRoles" adm
                WHERE adm.association_id = ar.association_id
                AND adm.user_id = $1
                AND adm.role = 'admin'
                AND adm.pending = FALSE
            )
            "#,
            admin_user_id
        ).fetch_one(db).await?.unwrap_or(0);

        // -------- pagination ----------
        let page_size = page_size.max(1);
        let total_pages = (total_size + page_size - 1) / page_size;
        page = page.max(1).min(total_pages.max(1));
        let offset = (page - 1) * page_size;

        // -------- DATA ----------
        let items: Vec<PendingMember> = sqlx::query_as!(
            PendingMember,
            r#"
            SELECT
            u.id, u.name, u.birthday, u.address, u.activity,
            u.email, u.personal_phone, u.commercial_phone,
            u.uses_whatsapp, u.identities, u.profile_url,
            ar.association_id,
            a.name AS "association_name!",
            ar.created_at AS "requested_at!"
            FROM "AssociationRoles" ar
            JOIN "User" u ON u.id = ar.user_id
            JOIN "Association" a ON a.id = ar.association_id
            WHERE COALESCE(u.deleted, FALSE) = FALSE
            AND COALESCE(a.deleted, FALSE) = FALSE
            AND ar.role = 'member'
            AND ar.pending = TRUE
            AND u.id <> $1
            AND EXISTS (
                SELECT 1 FROM "AssociationRoles" adm
                WHERE adm.association_id = ar.association_id
                AND adm.user_id = $1
                AND adm.role = 'admin'
                AND adm.pending = FALSE
            )
            ORDER BY ar.created_at DESC, ar.association_id, u.id
            LIMIT $2 OFFSET $3
            "#,
            admin_user_id, page_size, offset
        ).fetch_all(db).await?;

        Ok(PendingMembersPage {
            total_size,
            page,
            page_size,
            total_pages,
            has_previous_page: page > 1,
            has_next_page: page < total_pages,
            items,
        })
    }

    pub async fn toggle_approve(
        db: &DB,
        user_id: &Uuid,
        association_id: &Uuid,
    ) -> Result<bool, anyhow::Error> {
        let mut tx = db.begin().await?;
        let pending: bool = sqlx::query_scalar!(
            r#"
                UPDATE "AssociationRoles"
                SET pending = not pending
                WHERE user_id = $1 AND association_id = $2 AND role = 'member'
                RETURNING pending
                "#,
            user_id,
            association_id,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(pending)
    }

    pub async fn update(db: &DB, user: UserUpdate) -> Result<User, anyhow::Error> {
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = db.begin().await?;
        let new_user = sqlx::query_as!(
            User,
            r#"
            UPDATE "User" SET
                name = COALESCE($1, name),
                birthday = COALESCE($2, birthday),
                address = COALESCE($3, address),
                activity = COALESCE($4, activity),
                personal_phone = COALESCE($5, personal_phone),
                commercial_phone = COALESCE($6, commercial_phone),
                uses_whatsapp = COALESCE($7, uses_whatsapp),
                identities = COALESCE($8, identities),
                profile_url = COALESCE($9, profile_url),
                deleted = COALESCE($10, deleted)
            WHERE id = $11
            RETURNING *
            "#,
            user.name,
            user.birthday,
            user.address,
            user.activity,
            user.personal_phone,
            user.commercial_phone,
            user.uses_whatsapp,
            user.identities,
            user.profile_url,
            user.deleted,
            user.id
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(new_user)
    }
}
