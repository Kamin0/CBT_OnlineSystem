use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::achievements;
use crate::schema::ranks;
use crate::schema::user_achievements;
use crate::schema::users;
use crate::schema::sessions;
use crate::schema::users::username;

#[derive(Debug, Serialize, Deserialize, diesel::Queryable, diesel::Insertable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub kda: f32,
    pub role_id: Uuid, // Assuming you have a separate table for roles
    pub rank_id: Uuid, // Assuming you have a separate table for ranks
}

#[derive(Debug, Serialize, Deserialize, diesel::Queryable, diesel::Insertable)]
#[diesel(table_name = user_achievements)]
pub struct UserAchievement {
    pub user_id: Uuid,
    pub achievement_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, diesel::Queryable, diesel::Insertable)]
#[diesel(table_name = achievements)]
pub struct Achievement {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize, diesel::Queryable, diesel::Insertable)]
#[diesel(table_name = ranks)]
pub struct Rank {
    pub id: Uuid,
    pub name: String,
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize, diesel::Queryable, diesel::Insertable,Clone)]
#[diesel(table_name = sessions)]
pub struct DBSession {
    pub id: Uuid,
    pub average_kda: f32,
    pub average_rank: Uuid,
    pub is_empty: bool,
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role_name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub server_address: String,
    pub players: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub session_id: Uuid,
    pub server_address: String
}

#[derive(Debug, Deserialize)]
pub struct AchievementValidation {
    pub username: String,
    pub achievement_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct KdaUpdate {
    pub username: String,
    pub new_kda: f32,
}

#[derive(Debug, Deserialize)]
pub struct RankUpdate {
    pub username: String,
    pub new_rank_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectSession {
    pub session_id: Uuid,
    pub username: String,
}
