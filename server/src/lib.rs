use sqlx::{Pool, Postgres};

pub mod association;
pub mod graphql;
pub mod user;

pub type DB = Pool<Postgres>;
