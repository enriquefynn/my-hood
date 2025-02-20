use async_graphql::{Context, InputObject, Object};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{association::model::Association, relations::model::Relations, DB};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct User {
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
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, InputObject, Deserialize)]
pub struct UserInput {
    // Set as pub so we can reuse this struct for Oauth.
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

    pub async fn associations(&self, ctx: &Context<'_>) -> Result<Vec<Association>, anyhow::Error> {
        let pool = ctx.data::<PgPool>().unwrap();
        let mut tx = pool.begin().await?;
        let associations = sqlx::query_as!(
            Association,
            r#"SELECT a.* FROM "Association" a 
        INNER JOIN "UserAssociation" ua ON a.id = ua.association_id WHERE ua.user_id = $1"#,
            self.id
        )
        .fetch_all(&mut *tx)
        .await?;
        Ok(associations)
    }

    pub async fn is_admin(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        Relations::is_admin(ctx, self.id, association_id).await
    }

    pub async fn is_treasurer(
        &self,
        ctx: &Context<'_>,
        association_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        Relations::is_treasurer(ctx, self.id, association_id).await
    }
}

impl User {
    pub async fn create(db: &DB, user: UserInput) -> Result<User, anyhow::Error> {
        let mut tx = db.begin().await?;

        let user = sqlx::query_as!(
            User,
            r#"INSERT INTO "User" (name, birthday, address, activity, email, personal_phone,
                commercial_phone, uses_whatsapp, identities, profile_url)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *"#,
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
        let mut tx = db.begin().await?;
        let user = sqlx::query_as!(User, r#"SELECT * FROM "User" WHERE id = $1"#, id)
            .fetch_one(&mut *tx)
            .await?;
        Ok(user)
    }

    pub async fn read_one_by_email(db: &DB, email: &str) -> Result<Option<User>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let user = sqlx::query_as!(User, r#"SELECT * FROM "User" WHERE email = $1"#, email)
            .fetch_optional(&mut *tx)
            .await?;
        Ok(user)
    }

    pub async fn read_all(db: DB) -> Result<Vec<User>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let users = sqlx::query_as!(User, r#"SELECT * FROM "User""#)
            .fetch_all(&mut *tx)
            .await?;
        Ok(users)
    }
}
