use async_graphql::{Context, InputObject, Object};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    association::model::Association,
    relations::model::{Relations, Role},
    DB,
};

#[derive(Debug, FromRow, Deserialize, Serialize, Clone)]
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
    pub async fn create(db: &DB, user: UserInput) -> Result<User, anyhow::Error> {
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
