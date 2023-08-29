use crate::{
    association::graphql::{AssociationMutation, AssociationQuery},
    config::Config,
    relations::graphql::RelationsMutation,
    transaction::graphql::{TransactionMutation, TransactionQuery},
    user::graphql::{UserMutation, UserQuery},
    DB,
};

use async_graphql::{EmptySubscription, MergedObject, Schema};

#[derive(MergedObject, Default)]
pub struct Query(UserQuery, AssociationQuery, TransactionQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(
    UserMutation,
    AssociationMutation,
    TransactionMutation,
    RelationsMutation,
);

pub fn get_schema(db: DB, config: Config) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db)
        .data(config)
        .finish()
}
