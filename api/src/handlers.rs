use std::sync::{Arc, Mutex};
use actix_web::{HttpResponse, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{NewUser, User};
use crate::schema::{roles, users};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    // Add more claims as needed
}

pub async fn register_user(user_data: web::Json<NewUser>, db: web::Data<Arc<Mutex<PgConnection>>>) -> HttpResponse {
    let user_data = user_data.into_inner();

    // Hash password
    let hashed_password = hash(&user_data.password, DEFAULT_COST).unwrap();

    // Find role_id by role name
    let role_id: Uuid = match roles::table
        .select(roles::id)
        .filter(roles::name.eq(&user_data.role_name))
        .first(&mut *db.lock().unwrap()) // Lock the Mutex and unwrap to get the PgConnection
    {
        Ok(id) => id,
        Err(_) => {
            // If the role does not exist, you might want to handle this case appropriately
            return HttpResponse::BadRequest().body("Invalid role name provided");
        }
    };

    // Generate salt
    let salt = Uuid::new_v4().to_string();

    // Create new user
    let new_user = User {
        id: Uuid::new_v4(),
        username: user_data.username,
        email: user_data.email,
        password: hashed_password,
        salt: salt.clone(),
        role_id,
    };

    // Insert new user into the database
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut *db.lock().unwrap()) // Lock the Mutex and unwrap to get the PgConnection
        .expect("Error inserting user into database");

    HttpResponse::Ok().body("User registered successfully")
}

pub async fn login_user(user_data: web::Json<NewUser>, db: web::Data<Arc<Mutex<PgConnection>>>) -> HttpResponse {
    let user_data = user_data.into_inner();

    // Retrieve user from database
    let user: User = users::table
        .filter(users::username.eq(&user_data.username))
        .first(&mut *db.lock().unwrap())
        .expect("Error retrieving user from database");

    // Verify password
    let is_valid_password = verify(&user_data.password, &user.password).unwrap();
    if is_valid_password {
        // Generate JWT token
        let claims = Claims {
            sub: user.id.to_string(),
            role: user.role_id.to_string(),
            // Add more claims as needed
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();

        HttpResponse::Ok().json(token)
    } else {
        HttpResponse::Unauthorized().finish()
    }
}

