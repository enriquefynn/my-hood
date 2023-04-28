use async_graphql::{Context, FieldResult, Object};

use crate::{
    db::Database,
    schema::{CreateUpdateUser, FetchUser, User},
};

pub struct Query;

#[Object(extends)]
impl Query {
    // Query user.
    async fn user(&self, ctx: &Context<'_>, input: FetchUser) -> FieldResult<User> {
        let db = ctx.data_unchecked::<Database>();
        let user = db.get_user(input.id).await?;
        Ok(user)
    }
}

pub struct Mutation;

#[Object(extends)]
impl Mutation {
    // Mutate user.
    async fn create_user(&self, ctx: &Context<'_>, input: CreateUpdateUser) -> FieldResult<User> {
        let db = ctx.data_unchecked::<Database>();
        let user = if let Some(_) = &input.id {
            // ID present, we alter the user.
            db.update_user(input).await?
        } else {
            db.create_user(input).await?
        };
        Ok(user)
    }
}
