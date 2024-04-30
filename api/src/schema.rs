use diesel::{allow_tables_to_appear_in_same_query, joinable, table};
// schema.rs
table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
        password -> Varchar,
        salt -> Varchar,
        kda -> Float,
        nb_games -> Int4,
        role_id -> Uuid,
        rank_id -> Uuid,
    }
}

table! {
    roles (id) {
        id -> Uuid,
        name -> Varchar,
    }
}

table! {
    user_achievements (user_id, achievement_id) {
        user_id -> Uuid,
        achievement_id -> Uuid,
    }
}

table! {
    achievements (id) {
        id -> Uuid,
        name -> Varchar,
        description -> Text,
        image_url -> VarChar,
    }
}

table! {
    ranks (id) {
        id -> Uuid,
        name -> Varchar,
        image_url -> VarChar,
    }
}

table! {
    sessions (id) {
        id -> Uuid,
        average_kda -> Float,
        average_rank -> Uuid,
        is_empty -> Bool,
    }
}

joinable!(users -> roles (role_id));
joinable!(users -> ranks (rank_id));
joinable!(user_achievements -> users (user_id));
joinable!(user_achievements -> achievements (achievement_id));
joinable!(sessions -> ranks (average_rank));

allow_tables_to_appear_in_same_query!(
    users,
    roles,
);

allow_tables_to_appear_in_same_query!(
    users,
    ranks,
);

allow_tables_to_appear_in_same_query!(
    user_achievements,
    achievements,
);
