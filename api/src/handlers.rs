use actix_web::{HttpRequest, HttpResponse, web};
use actix_web::web::Data;
use bcrypt::{DEFAULT_COST, hash_with_salt, verify};
use chrono::{Duration, Utc};
use diesel::{PgConnection, QueryDsl, r2d2, RunQueryDsl};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use rand::random;
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{AchievementValidation, LoginUser, NewUser, Session, User, UserAchievement};
use crate::schema::{roles, user_achievements, users};

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

pub async fn validate_achievement(user_data: web::Json<AchievementValidation>, pool:Data<DbPool>, http_request: HttpRequest) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(http_request, "server".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Extract user data from request
            let user_data = user_data.into_inner();

            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

            // Check if user exists
            let user_id: Uuid = match users::table
                .select(users::id)
                .filter(users::username.eq(&user_data.username))
                .first(&mut conn)
            {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::BadRequest().body("Invalid username");
                }
            };

            let user_achievement = UserAchievement {
                user_id,
                achievement_id: user_data.achievement_id,
            };

            diesel::insert_into(user_achievements::table)
                .values(&user_achievement)
                .execute(&mut conn)
                .expect("Error inserting user achievement into database");

            HttpResponse::Ok().body("Achievement validated successfully")
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }

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
    // Extract JWT token from request headers
    let token_validation = validate_token(req, "server".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
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
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

pub(crate) async fn request_session(
    req: HttpRequest,
    redis: web::Data<Client>,
) -> HttpResponse {
    // Extract JWT token from request headers
    let token_validation = validate_token(req, "client".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Create connection to Redis
            match redis.get_ref().get_multiplexed_async_connection().await {
                Ok(mut con) => {
                    // Retrieve session data from Redis
                    let session_data: Result<String, RedisError> = con.get("session").await;
                    match session_data {
                        Ok(data) => {
                            let session: Session = serde_json::from_str(&data).unwrap();
                            HttpResponse::Ok().json(session)
                        }
                        Err(_) => HttpResponse::NotFound().body("Session not found"),
                    }
                }
                Err(_) => HttpResponse::InternalServerError().body("Failed to connect to Redis"),
            }
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

fn validate_token(req: HttpRequest, role_value : String) -> i32 {
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

    return match token_data {
        Ok(claims) => {
            if claims.claims.role == role_value {
                0
            } else {
                2
            }
        }
        Err(_) => 1,
    }
}
