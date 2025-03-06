use crate::{
    association::graphql::{AssociationMutation, AssociationQuery},
    config::Config,
    relations::graphql::RelationsMutation,
    token::Claims,
    transaction::graphql::{TransactionMutation, TransactionQuery},
    user::graphql::{UserMutation, UserQuery},
    DB,
};

use async_graphql::{EmptySubscription, MergedObject, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{response::IntoResponse, Extension};

#[derive(MergedObject, Default)]
pub struct Query(UserQuery, AssociationQuery, TransactionQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(
    UserMutation,
    AssociationMutation,
    TransactionMutation,
    RelationsMutation,
);
pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn get_schema(db: DB, config: Config) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db)
        .data(config)
        .finish()
}

pub async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    claims: Claims,
    req: GraphQLRequest,
) -> impl IntoResponse {
    // // Turn the incoming request into an async-graphql `Request`
    let mut request = req.into_inner();
    // // Insert claims so that resolvers can access them via Context
    request = request.data(claims);

    let response = schema.execute(request).await;
    GraphQLResponse::from(response)
}
