/// Automatically generates structures to be used with GraphQL and sqlx
/// The first argument is used for create and update fields and
/// the second is used for querying.
#[macro_export]
macro_rules! create_update_object {
    ($create_update_name:ident, $struct_name:ident {
        $(
            $(#[$attr:meta])*
            $field:ident : $type:ty
        ),*
        $(,)?
    }) => {
        #[derive(Debug, SimpleObject, sqlx::FromRow)]
        pub struct $struct_name {
            pub id: String,
            pub updated_at: chrono::NaiveDateTime,
            $(
                $(#[$attr])*
                pub $field: $type,
            )*
        }
        #[derive(Debug, InputObject)]
        pub struct $create_update_name {
            pub id: Option<String>,
            $(
                $(#[$attr])*
                pub $field: $type,
            )*
        }
    };
}
