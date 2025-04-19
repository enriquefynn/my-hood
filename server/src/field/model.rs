use async_graphql::{Context, InputObject, Object, SimpleObject};
use bigdecimal::BigDecimal;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DB;

use super::rules::ReservationRules;

#[derive(Debug, FromRow, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub id: Uuid,
    pub association_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub reservation_rules: Option<String>,
    // Latitude and longitude of the field.
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
    pub deleted: Option<bool>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, InputObject, Deserialize)]
pub struct FieldInput {
    pub association_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub reservation_rules: Option<String>,
    // Latitude and longitude of the field.
    pub latitude: BigDecimal,
    pub longitude: BigDecimal,
}

#[derive(InputObject)]
pub struct FieldUpdate {
    pub id: Uuid,
    pub association_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub reservation_rules: Option<String>,
    // Latitude and longitude of the field.
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub deleted: Option<bool>,
}

#[derive(SimpleObject, Debug, FromRow, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldReservation {
    pub id: Uuid,
    pub field_id: Uuid,
    pub user_id: Uuid,
    pub description: Option<String>,
    pub start_date: chrono::DateTime<Utc>,
    pub end_date: chrono::DateTime<Utc>,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Debug, InputObject, Deserialize)]
pub struct FieldReservationInput {
    pub field_id: Uuid,
    pub user_id: Uuid,
    pub description: Option<String>,
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
}

#[derive(InputObject)]
pub struct FieldReservationUpdate {
    pub id: Uuid,
    pub field_id: Uuid,
    pub user_id: Uuid,
    pub description: Option<String>,
    pub start_date: Option<chrono::NaiveDateTime>,
    pub end_date: Option<chrono::NaiveDateTime>,
    pub deleted: Option<bool>,
}

#[Object]
impl Field {
    pub async fn id(&self) -> Uuid {
        self.id
    }

    pub async fn association_id(&self) -> Uuid {
        self.association_id
    }

    pub async fn name(&self) -> String {
        self.name.clone()
    }

    pub async fn description(&self) -> Option<String> {
        self.description.clone()
    }

    pub async fn reservation_rules(&self) -> Option<String> {
        self.reservation_rules.clone()
    }

    pub async fn latitude(&self) -> BigDecimal {
        self.latitude.clone()
    }

    pub async fn longitude(&self) -> BigDecimal {
        self.longitude.clone()
    }

    pub async fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    pub async fn updated_at(&self) -> chrono::NaiveDateTime {
        self.updated_at
    }

    async fn reservations(
        &self,
        ctx: &Context<'_>,
        from_date_time: chrono::DateTime<chrono::Utc>,
        to_date_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<FieldReservation>, anyhow::Error> {
        let pool = ctx.data::<DB>().unwrap();

        if to_date_time - from_date_time >= chrono::Duration::days(30) {
            return Err(anyhow::Error::msg("Date range too large").into());
        }

        let field_reservations = sqlx::query_as!(
            FieldReservation,
            r#"
            SELECT * FROM "FieldReservation" WHERE field_id = $1 AND deleted = false AND start_date >= $2 AND end_date <= $3
            "#,
            self.id,
            from_date_time,
            to_date_time
        )
        .fetch_all(&*pool)
        .await?;
        Ok(field_reservations)
    }
}

impl Field {
    pub async fn create(db: &DB, field: FieldInput) -> Result<Field, anyhow::Error> {
        let mut tx = db.begin().await?;
        let rules_opt = &field.reservation_rules;
        rules_opt
            .clone()
            .map(|json| {
                ReservationRules::from_json(&json)
                    .map_err(|e| anyhow::anyhow!("Failed to parse reservation rules: {}", e))
            })
            .transpose()?;

        let field = sqlx::query_as!(
            Field,
            r#"
            INSERT INTO "Field" (association_id, name, description, reservation_rules, latitude, longitude)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            field.association_id,
            field.name,
            field.description,
            field.reservation_rules,
            field.latitude,
            field.longitude,
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(field)
    }

    pub async fn get(db: &DB, id: &Uuid) -> Result<Field, anyhow::Error> {
        let field = sqlx::query_as!(Field, r#"SELECT * FROM "Field" WHERE id = $1"#, id)
            .fetch_one(&*db)
            .await?;
        Ok(field)
    }
}

impl FieldReservation {
    pub async fn get(
        db: &DB,
        field_reservation_id: &Uuid,
    ) -> Result<FieldReservation, anyhow::Error> {
        let field_reservation = sqlx::query_as!(
            FieldReservation,
            r#"SELECT * FROM "FieldReservation" WHERE id = $1"#,
            field_reservation_id
        )
        .fetch_one(&*db)
        .await?;
        Ok(field_reservation)
    }

    pub async fn delete(
        db: &DB,
        field_reservation_id: &Uuid,
    ) -> Result<FieldReservation, anyhow::Error> {
        let mut tx = db.begin().await?;
        let field_reservation = sqlx::query_as!(
            FieldReservation,
            r#"
            UPDATE "FieldReservation" SET deleted = true WHERE id = $1 RETURNING *
            "#,
            field_reservation_id
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(field_reservation)
    }

    pub async fn create(
        db: &DB,
        user_id: &Uuid,
        field: &Field,
        field_reservation: FieldReservationInput,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<FieldReservation, anyhow::Error> {
        let rules = &field.reservation_rules;

        if let Some(rules) = rules {
            let rules: ReservationRules = serde_json::from_str(&rules)?;
            rules
                .can_reserve(
                    db,
                    user_id,
                    field_reservation.start_date,
                    field_reservation.end_date,
                    now,
                )
                .await?;
        }

        let mut tx = db.begin().await?;
        let field_reservation = sqlx::query_as!(
            FieldReservation,
            r#"
            INSERT INTO "FieldReservation" (field_id, user_id, description, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            field.id,
            field_reservation.user_id,
            field_reservation.description,
            field_reservation.start_date,
            field_reservation.end_date
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;

        Ok(field_reservation)
    }
}
