use sqlx::{Pool, Postgres};
use user::model::User;

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
