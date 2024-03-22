use async_graphql::{Context, InputObject, Object};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{relations::model::Relations, transaction::model::Transaction, user::model::User, DB};

#[derive(FromRow, Deserialize, Serialize)]
pub struct Association {
    pub id: Uuid,
    pub name: String,
    pub neighborhood: String,
    pub country: String,
    pub state: String,
    pub address: String,
    pub identity: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(InputObject)]
pub struct AssociationInput {
    name: String,
    neighborhood: String,
    country: String,
    state: String,
    address: String,
    identity: Option<String>,
}

#[Object]
impl Association {
    pub async fn id(&self) -> Uuid {
        self.id
    }

    pub async fn name(&self) -> String {
        self.name.to_owned()
    }

    pub async fn neighborhood(&self) -> String {
        self.neighborhood.to_owned()
    }

    pub async fn country(&self) -> String {
        self.country.to_owned()
    }

    pub async fn state(&self) -> String {
        self.state.to_owned()
    }

    pub async fn address(&self) -> String {
        self.address.to_owned()
    }

    pub async fn identity(&self) -> Option<String> {
        self.identity.to_owned()
    }

    pub async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>, anyhow::Error> {
        let pool = ctx.data::<PgPool>().unwrap();
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;
        let users = sqlx::query_as!(
            User,
            r#"SELECT a.* FROM "User" a 
        INNER JOIN UserAssociation ua ON a.id = ua.user_id WHERE ua.association_id = $1"#,
            self.id
        )
        .fetch_all(&mut *tx)
        .await?;
        Ok(users)
    }

    async fn transactions(
        &self,
        ctx: &Context<'_>,
        from_date: chrono::NaiveDate,
        to_date: chrono::NaiveDate,
    ) -> Result<Vec<Transaction>, anyhow::Error> {
        let pool = ctx.data::<PgPool>().unwrap();
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let transactions = sqlx::query_as!(
            Transaction,
            r#"SELECT * FROM Transaction WHERE association_id = $1 AND reference_date >= $2 AND reference_date < $3"#,
            self.id,
            from_date,
            to_date
        )
        .fetch_all(&mut *tx)
        .await?;

        Ok(transactions)
    }

    pub async fn is_admin(&self, ctx: &Context<'_>, user_id: Uuid) -> Result<bool, anyhow::Error> {
        Relations::is_admin(ctx, user_id, self.id).await
    }

    pub async fn is_treasurer(
        &self,
        ctx: &Context<'_>,
        user_id: Uuid,
    ) -> Result<bool, anyhow::Error> {
        Relations::is_treasurer(ctx, user_id, self.id).await
    }
}

impl Association {
    pub async fn create(
        db: &DB,
        association: AssociationInput,
    ) -> Result<Association, anyhow::Error> {
        let mut tx = db.begin().await?;

        let user = sqlx::query_as!(
            Association,
            r#"INSERT INTO Association (name, neighborhood, country, state, address,
                identity)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *"#,
            association.name,
            association.neighborhood,
            association.country,
            association.state,
            association.address,
            association.identity,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(user)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<Association, anyhow::Error> {
        let mut tx = db.begin().await?;
        let user = sqlx::query_as!(
            Association,
            r#"SELECT * FROM Association WHERE id = $1"#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        Ok(user)
    }

    pub async fn read_all(db: DB) -> Result<Vec<Association>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let users = sqlx::query_as!(Association, r#"SELECT * FROM Association"#)
            .fetch_all(&mut *tx)
            .await?;
        Ok(users)
    }
}
