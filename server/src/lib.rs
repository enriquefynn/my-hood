use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use graphql::{Mutation, Query};
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

// Graphql entry.
pub async fn index(
    schema: Schema<Query, Mutation, EmptySubscription>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
