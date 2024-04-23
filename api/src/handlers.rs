use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::web::Data;
use bcrypt::{DEFAULT_COST, hash_with_salt, verify};
use diesel::{PgConnection, QueryDsl, r2d2, RunQueryDsl};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use rand::random;
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::models::{LoginUser, NewUser, Session, User};
use crate::schema::{roles, users};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
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

    //Get the role name from the roles database
    let role_name: String = match roles::table
        .select(roles::name)
        .filter(roles::id.eq(&user.role_id))
        .first(&mut conn)
    {
        Ok(name) => name,
        Err(_) => {
            return HttpResponse::BadRequest().body("Invalid role id");
        }
    };

    // Verify password
    let is_valid_password = verify(&user_data.password, &user.password).unwrap();
    if is_valid_password {
        // Generate JWT token
        let expiration = Utc::now() + Duration::seconds(3600);
        let claims = Claims {
            sub: user.id.to_string(),
            role: role_name,
            exp: expiration.timestamp() as usize,
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();

        HttpResponse::Ok().json(token)
    } else {
        HttpResponse::Unauthorized().body("Invalid username or password")
    }
}

pub async fn register_session(
    req: HttpRequest,
    session: web::Json<Session>,
    redis: Data<Client>
) -> HttpResponse {
    // Extract user id from JWT token
    let token = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .unwrap()
        .to_str()
        .unwrap();
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    match token_data {
        Ok(claims) => {
            // Check if user has permission to register session
            //TODO : check expiration time
            if claims.claims.role == "server" {
                // Create connection to Redis
                match redis.get_ref().get_multiplexed_async_connection().await {
                    Ok(mut con) => {
                        // Store session data in Redis
                        let _: Result<(), RedisError> = con
                            .set("session", serde_json::to_string(&session.into_inner()).unwrap())
                            .await;
                        HttpResponse::Ok().body("Session registered successfully")
                    }
                    Err(_) => HttpResponse::InternalServerError().body("Failed to connect to Redis"),
                }
            } else {
                HttpResponse::Forbidden().body("Permission denied")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Unauthorized"),
    }
}

pub(crate) async fn request_session(
    req: HttpRequest,
    redis: web::Data<Client>,
) -> HttpResponse {
    // Extract JWT token from request headers
    let token = req
        .headers()
        .get(actix_web::http::header::AUTHORIZATION)
        .unwrap()
        .to_str()
        .unwrap();

    // Decode JWT token
    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret("secret".as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    match token_data {
        Ok(claims) => {
            // Check if user has permission to request session
            //TODO : check expiration time
            if claims.claims.role == "client" {
                // Create connection to Redis
                match redis.get_ref().get_multiplexed_async_connection().await {
                    Ok(mut con) => {
                        // Retrieve session data from Redis
                        match con.get::<_,String>("session").await {
                            Ok(data) => HttpResponse::Ok().body(data),
                            Err(_) => HttpResponse::InternalServerError().body("Error retrieving session data"),
                        }
                    }
                    Err(_) => HttpResponse::InternalServerError().body("Failed to connect to Redis"),
                }
            } else {
                HttpResponse::Forbidden().body("Permission denied")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Unauthorized"),
    }
}


