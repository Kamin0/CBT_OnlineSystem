use diesel::{allow_tables_to_appear_in_same_query, joinable, table};
// schema.rs
table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        salt -> Varchar,
        role_id -> Uuid,
    }
}

table! {
    roles (id) {
        id -> Uuid,
        name -> Varchar,
    }
}

joinable!(users -> roles (role_id));

allow_tables_to_appear_in_same_query!(
    users,
    roles,
);
