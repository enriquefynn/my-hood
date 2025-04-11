use chrono::Datelike;
use serde::{Deserialize, Serialize};

use crate::{field::model::FieldReservation, DB};

#[derive(Debug, Serialize, Deserialize)]
pub enum ReservationPeriod {
    Daily,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReservationRules {
    reservations_start_at_time_utc: chrono::NaiveTime,
    max_duration_minutes: u32,
    max_reservations_per_period: u32,
    reservation_period: ReservationPeriod,
}

impl ReservationRules {
    pub fn new(
        reservations_start_at_time_utc: chrono::NaiveTime,
        max_duration_minutes: u32,
        max_reservations_per_period: u32,
        reservation_period: ReservationPeriod,
    ) -> Self {
        Self {
            reservations_start_at_time_utc,
            max_duration_minutes,
            max_reservations_per_period,
            reservation_period,
        }
    }
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub async fn can_reserve(
        &self,
        db: &DB,
        start_date_time: chrono::DateTime<chrono::Utc>,
        end_date_time: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), anyhow::Error> {
        let now = chrono::Utc::now();
        match self.reservation_period {
            ReservationPeriod::Daily => {
                if now.day() != start_date_time.day() {
                    return Err(anyhow::anyhow!("Reservations can only be made for today"));
                }
                if now.time() < self.reservations_start_at_time_utc {
                    return Err(anyhow::anyhow!(
                        "Reservations can only be made after {}",
                        self.reservations_start_at_time_utc
                    ));
                }
                let duration_minutes = (end_date_time - start_date_time).num_minutes() as u32;
                if duration_minutes > self.max_duration_minutes {
                    return Err(anyhow::anyhow!(
                        "Reservations can only be made for a maximum of {} minutes",
                        self.max_duration_minutes
                    ));
                }

                let reservations = sqlx::query_as!(
                    FieldReservation,
                    r#"SELECT * FROM "FieldReservation" WHERE start_date >= $1 AND end_date <= $2
                    ORDER BY start_date ASC, end_date ASC;"#,
                    start_date_time,
                    end_date_time,
                )
                .fetch_optional(&*db)
                .await?;
                if reservations.is_some() {
                    return Err(anyhow::anyhow!("Field overlaps with another reservation"));
                }
            }
        }
        Ok(())
    }
}
