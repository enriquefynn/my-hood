use async_graphql::{EmptySubscription, Schema};
use sqlx::{Pool, Postgres};

use self::graphql::{Mutation, Query};

pub mod graphql;
pub mod model;

pub(crate) type DB = Pool<Postgres>;

pub fn get_user_schema(db: DB) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(db)
        .finish()
}
