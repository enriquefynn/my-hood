use sqlx::{Pool, Postgres};

pub mod association;
pub mod config;
pub mod error;
pub mod graphql;
// pub mod oauth;
pub mod relations;
pub mod token;
pub mod transaction;
pub mod user;

pub type DB = Pool<Postgres>;
