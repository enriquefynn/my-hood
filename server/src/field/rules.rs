use chrono::NaiveTime;
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
        user_id: &uuid::Uuid,
        start_date_time: chrono::DateTime<chrono::Utc>,
        end_date_time: chrono::DateTime<chrono::Utc>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), anyhow::Error> {
        match self.reservation_period {
            ReservationPeriod::Daily => {
                if now.date_naive() != start_date_time.date_naive() {
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
                    r#"SELECT * FROM "FieldReservation" WHERE deleted = false AND start_date >= $1 AND end_date <= $2
                    ORDER BY start_date ASC, end_date ASC;"#,
                    start_date_time,
                    end_date_time,
                )
                .fetch_optional(&*db)
                .await?;
                if reservations.is_some() {
                    return Err(anyhow::anyhow!("Field overlaps with another reservation"));
                }

                let today_start = now
                    .with_time(NaiveTime::from_hms_opt(0, 0, 0).expect("Should be valid time"))
                    .unwrap();
                let today_end = now
                    .with_time(NaiveTime::from_hms_opt(23, 59, 59).expect("Should be valid time"))
                    .unwrap();
                let user_reservations = sqlx::query!(
                    r#"SELECT count(*) FROM "FieldReservation" WHERE deleted = false AND user_id = $1 AND start_date >= $2 AND end_date <= $3"#,
                    user_id,
                    today_start,
                    today_end,
                ).fetch_optional(&*db).await?;

                if let Some(user_reservations) = user_reservations {
                    if user_reservations.count.unwrap_or(0) as u32
                        >= self.max_reservations_per_period
                    {
                        return Err(anyhow::anyhow!(
                            "User has reached the maximum number of reservations for today"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
