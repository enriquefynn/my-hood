use crate::{
    association::graphql::{UserMutation, UserQuery},
    user::graphql::{AssociationMutation, AssociationQuery},
    DB,
};

use async_graphql::{EmptySubscription, MergedObject, Schema};

#[derive(MergedObject, Default)]
pub struct Query(UserQuery, AssociationQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(UserMutation, AssociationMutation);

pub fn get_schema(db: DB) -> Schema<Query, Mutation, EmptySubscription> {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db)
        .finish()
}
