use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};

pub mod association;
pub mod config;
pub mod error;
pub mod field;
pub mod graphql;
pub mod oauth;
pub mod relations;
pub mod token;
pub mod transaction;
pub mod user;

pub type DB = Pool<Postgres>;

#[async_trait]
pub trait Clock: Sync + Send {
    fn now(&self) -> DateTime<Utc>;
}

pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
