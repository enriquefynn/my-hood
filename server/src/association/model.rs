use async_graphql::{Context, InputObject, Object, SimpleObject};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, QueryBuilder};
use uuid::Uuid;

use crate::{
    field::model::Field,
    relations::model::{Relations, Role},
    transaction::model::Transaction,
    user::model::User,
    DB,
};

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Association {
    pub id: Uuid,
    pub name: String,
    pub neighborhood: String,
    pub country: String,
    pub state: String,
    pub address: String,
    pub identity: Option<String>,
    pub public: bool,
    pub deleted: Option<bool>,
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
    public: Option<bool>,
    deleted: Option<bool>,
    identity: Option<String>,
}

#[derive(Debug)]
pub struct AssocFilter {
    pub search:      Option<String>,
    pub member_only: bool,
    pub pending_only: bool,
    pub user_id:     Option<Uuid>, // Some(uuid) if member_only = true
    pub page:        i64,
    pub page_size:   i64,
}

#[derive(SimpleObject)]
pub struct AssociationsPage {
    pub total_size: i64,
    pub page:       i64,
    pub page_size:  i64,
    pub total_pages: i64,
    pub has_previous_page: bool,
    pub has_next_page:     bool,
    pub items: Vec<Association>,
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

    pub async fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    pub async fn updated_at(&self) -> chrono::NaiveDateTime {
        self.updated_at
    }

    pub async fn members(&self, ctx: &Context<'_>, only_admin: Option<bool>) -> Result<Vec<User>, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let mut tx: sqlx::Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let only_admin = only_admin.unwrap_or(true);

        let users = sqlx::query_as!(
            User,
            r#"SELECT DISTINCT u.* FROM "User" u
                INNER JOIN "AssociationRoles" ar ON u.id = ar.user_id WHERE ar.association_id = $1 
                AND COALESCE(u.deleted, FALSE) = FALSE AND ($2::bool IS FALSE OR ar.role = 'admin')"#,
            self.id,
            only_admin
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

    pub async fn fields(&self, ctx: &Context<'_>) -> Result<Vec<Field>, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();
        let fields = sqlx::query_as!(
            Field,
            r#"SELECT * FROM "Field" WHERE association_id = $1"#,
            self.id
        )
        .fetch_all(&*pool)
        .await?;
        Ok(fields)
    }

    pub async fn balance(&self, ctx: &Context<'_>) -> Result<BigDecimal, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();

        let total = sqlx::query_scalar!(r#"
        SELECT COALESCE(SUM(amount), 0)::numeric 
            FROM "Transaction"
                WHERE association_id = $1"#,
            self.id
        )
        .fetch_one(pool)
        .await?;

        Ok(total.unwrap_or_else(|| BigDecimal::from(0)))
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

    pub async fn read_filtered_paginated(
        db: &DB,
        filter: AssocFilter,
    ) -> anyhow::Result<AssociationsPage> {
        let AssocFilter {
            search,
            member_only,
            pending_only,
            user_id,
            mut page,
            page_size,
        } = filter;

        let search_pat = search.map(|s| format!("%{}%", s));

        // -------- COUNT ----------
        let mut count_qb =
            QueryBuilder::new(r#"SELECT COUNT(DISTINCT a.id) FROM "Association" a"#);

        if pending_only {
            let uid = user_id.ok_or_else(|| anyhow::Error::msg("user_id required for pending"))?;
            count_qb
                .push(r#" JOIN "AssociationRoles" ar ON a.id = ar.association_id AND ar.user_id = "#)
                .push_bind(uid)
                .push(" WHERE ar.user_id IS NOT NULL")
                .push(" AND ar.pending = true");
        } else if member_only {
            let uid = user_id.ok_or_else(|| anyhow::Error::msg("user_id required"))?;
            count_qb
                .push(r#" JOIN "AssociationRoles" ar ON a.id = ar.association_id AND ar.user_id = "#)
                .push_bind(uid)
                .push(" WHERE ar.user_id IS NOT NULL and ar.pending = false");
        } else {
            count_qb.push(" WHERE a.public = TRUE");
        }
        if let Some(ref pat) = search_pat {
            count_qb.push(" AND a.name ILIKE ").push_bind(pat);
        }

        let total_size: i64 = count_qb
            .build_query_scalar()
            .fetch_one(db)
            .await?;

        // pagination
        let page_size = page_size.max(1);
        let total_pages = (total_size + page_size - 1) / page_size;
        page = page.max(1).min(total_pages.max(1));
        let offset = (page - 1) * page_size;

        // -------- DATA ------------
        let mut data_qb =
            QueryBuilder::new(r#"SELECT DISTINCT a.* FROM "Association" a"#);

        if pending_only {
            let uid = user_id.ok_or_else(|| anyhow::Error::msg("user_id required for pending"))?;
            data_qb
                .push(r#" JOIN "AssociationRoles" ar ON a.id = ar.association_id AND ar.user_id = "#)
                .push_bind(uid)
                .push(" WHERE ar.user_id IS NOT NULL")
                .push(" AND ar.pending = true");
        } else if member_only {
            let uid = user_id.unwrap();
            data_qb
                .push(r#" JOIN "AssociationRoles" ar ON a.id = ar.association_id AND ar.user_id = "#)
                .push_bind(uid)
                .push(" WHERE ar.user_id IS NOT NULL and ar.pending = false");
        } else {
            data_qb.push(" WHERE a.public = TRUE");
        }
        if let Some(ref pat) = search_pat {
            data_qb.push(" AND a.name ILIKE ").push_bind(pat);
        }
        data_qb
            .push(" ORDER BY a.name")
            .push(" LIMIT ").push_bind(page_size)
            .push(" OFFSET ").push_bind(offset);

        let items: Vec<Association> = data_qb
            .build_query_as()
            .fetch_all(db)
            .await?;

        Ok(AssociationsPage {
            total_size,
            page,
            page_size,
            total_pages,
            has_previous_page: page > 1,
            has_next_page: page < total_pages,
            items,
        })
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
