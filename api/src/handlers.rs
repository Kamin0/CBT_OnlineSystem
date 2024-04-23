use actix_web::{HttpResponse, web};
use actix_web::web::Data;
use bcrypt::{DEFAULT_COST, hash_with_salt, verify};
use diesel::{PgConnection, QueryDsl, r2d2, RunQueryDsl};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::random;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{LoginUser, NewUser, User};
use crate::schema::{roles, users};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    // Add more claims as needed
}

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn hello() -> HttpResponse {
    HttpResponse::Ok().body("Hello, world!")
}

pub async fn register_user(user_data: web::Json<NewUser>, pool:Data<DbPool>) -> HttpResponse {
    // Extract user data from request
    let user = user_data.into_inner();

    // Generate salt and hash password
    let salt: [u8; 16] = random();
    let hashed_password = hash_with_salt(&user.password,DEFAULT_COST,salt).expect("Failed to hash password");

    // Establish a database connection
    let mut conn = pool.get().expect("Couldn't get db connection from pool");

    // Find role_id by role name
    let role_id: Uuid = match roles::table
        .select(roles::id)
        .filter(roles::name.eq(&user.role_name))
        .first(&mut conn) // Lock the Mutex and unwrap to get the PgConnection
    {
        Ok(id) => id,
        Err(_) => {
            // If the role does not exist, you might want to handle this case appropriately
            return HttpResponse::BadRequest().body("Invalid role name provided");
        }
    };

    // Create new user
    let new_user = User {
        id: Uuid::new_v4(),
        username: user.username,
        email: user.email,
        password: hashed_password.to_string(),
        salt: salt.to_vec().iter().map(|b| format!("{:02x}", b)).collect::<String>(),
        role_id,
    };

    // Insert new user into the database
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn) // Lock the Mutex and unwrap to get the PgConnection
        .expect("Error inserting user into database");

    HttpResponse::Ok().body("User registered successfully")
}

pub async fn login_user(user_data: web::Json<LoginUser>, pool:Data<DbPool>) -> HttpResponse {
    let user_data = user_data.into_inner();

    let mut conn = pool.get().expect("Couldn't get db connection from pool");

    // Retrieve user from database
    let user: User = match users::table
        .filter(users::username.eq(&user_data.username))
        .first(&mut conn)
    {
        Ok(user) => user,
        Err(_) => {
            return HttpResponse::Unauthorized().body("Invalid username or password");
        }
    };

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
        HttpResponse::Unauthorized().body("Invalid username or password")
    }
}

