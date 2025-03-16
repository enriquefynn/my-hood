use async_graphql::{Context, InputObject, Object};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{
    relations::model::{ Relations, Role},
    transaction::model::Transaction,
    user::model::User,
    DB,
};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct Association {
    pub id: Uuid,
    pub name: String,
    pub neighborhood: String,
    pub country: String,
    pub state: String,
    pub address: String,
    pub identity: Option<String>,
    pub public: bool,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(InputObject)]
pub struct AssociationUpdate {
    pub name: Option<String>,
    pub neighborhood: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub address: Option<String>,
    pub identity: Option<String>,
    pub public: Option<bool>,
    pub deleted: Option<bool>,
}

#[derive(InputObject)]
pub struct AssociationInput {
    name: String,
    neighborhood: String,
    country: String,
    state: String,
    address: String,
    public: bool,
    deleted: bool,
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

    pub async fn public(&self) -> bool {
        self.public
    }

    pub async fn members(&self, ctx: &Context<'_>) -> Result<Vec<User>, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;
        let users = sqlx::query_as!(
            User,
            r#"SELECT u.* FROM "User" u
        INNER JOIN "AssociationRoles" ar ON u.id = ar.user_id WHERE ar.association_id = $1 AND ar.role = 'admin'"#,
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
        let pool = ctx.data::<DB>().unwrap();
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let transactions = sqlx::query_as!(
            Transaction,
            r#"SELECT * FROM "Transaction" WHERE association_id = $1 AND reference_date >= $2 AND reference_date < $3"#,
            self.id,
            from_date,
            to_date
        )
        .fetch_all(&mut *tx)
        .await?;

        Ok(transactions)
    }

    pub async fn is_member(&self, ctx: &Context<'_>, user_id: Uuid) -> Result<bool, anyhow::Error> {
        let member = Relations::get_role(ctx, &user_id, self.id, Role::Member).await?;
        Ok(member.is_some())
    }
}

impl Association {
    pub async fn create(
        db: &DB,
        user_id: Uuid,
        association: AssociationInput,
    ) -> Result<Association, anyhow::Error> {
        let mut tx = db.begin().await?;
        let association = sqlx::query_as!(
            Association,
            r#"INSERT INTO "Association" (name, neighborhood, country, state, address,
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

        let _association_admin = Relations::create_association_role(
            &mut *tx,
            user_id,
            association.id,
            Role::Admin,
            false,
            None,
        )
        .await?;
        let _user_association = Relations::create_association_role(
            &mut *tx,
            user_id,
            association.id,
            Role::Member,
            false,
            None,
        )
        .await?;
        tx.commit().await?;
        Ok(association)
    }

    pub async fn read_one(db: &DB, id: &Uuid) -> Result<Association, anyhow::Error> {
        let mut tx = db.begin().await?;
        let association = sqlx::query_as!(
            Association,
            r#"SELECT * FROM "Association" WHERE id = $1"#,
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        Ok(association)
    }

    pub async fn read_all(db: &DB) -> Result<Vec<Association>, anyhow::Error> {
        let mut tx = db.begin().await?;
        let associations = sqlx::query_as!(Association, r#"SELECT * FROM "Association""#)
            .fetch_all(&mut *tx)
            .await?;
        Ok(associations)
    }

    pub async fn update(
        db: &DB,
        id: &Uuid,
        association: AssociationUpdate,
    ) -> Result<Association, anyhow::Error> {
        let mut tx = db.begin().await?;

        let association = sqlx::query_as!(
            Association,
            r#"UPDATE "Association"
                SET name = COALESCE($1, name),
                    neighborhood = COALESCE($2, neighborhood),
                    country = COALESCE($3, country),
                    state = COALESCE($4, state),
                    address = COALESCE($5, address),
                    identity = COALESCE($6, identity),
                    public = COALESCE($7, public),
                    deleted = COALESCE($8, deleted)
                WHERE id = $9 RETURNING *"#,
            association.name,
            association.neighborhood,
            association.country,
            association.state,
            association.address,
            association.identity,
            association.public,
            association.deleted,
            id
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(association)
    }
}
