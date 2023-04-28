use async_graphql::{InputObject, SimpleObject};

use crate::create_update_object;

#[derive(InputObject)]
pub struct FetchUser {
    pub id: String,
}

create_update_object!(CreateUpdateUser, User {
    name: String,
    birthday: chrono::NaiveDateTime,
    address: String,
    activity: Option<String>,
    email: Option<String>,
    personal_phone: Option<String>,
    commercial_phone: Option<String>,
    #[graphql(default = false)]
    uses_whatsapp: bool,
    signed_at: chrono::NaiveDateTime,
    // Identities fields separated by comma.
    identities: String,
});

create_update_object!(CreateUpdateNeighborhood, Neighborhood { name: String });

create_update_object!(
    CreateUpdateRevenueExpense,
    RevenueExpense {
        description: String,
        amount: u32,
        date: chrono::NaiveDateTime,
    }
);

struct UserNeighborhood {
    neighborhood_id: String,
    user_id: String,
    is_admin: bool,
}
