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
use web::Json;

use crate::models::{Achievement, AchievementValidation, DBSession, KdaUpdate, LoginUser, NewUser, Rank, RankUpdate, Session, User, UserAchievement};
use crate::schema::{achievements, ranks, roles, sessions, user_achievements, users};

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

pub async fn register_user(user_data: Json<NewUser>, pool:Data<DbPool>) -> HttpResponse {
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

    //Find the id of the rank bronze
    let rank_id: Uuid = match ranks::table
        .select(ranks::id)
        .filter(ranks::name.eq("Bronze"))
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
        kda: 0.0,
        role_id,
        rank_id
    };

    // Insert new user into the database
    return match diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn) // Lock the Mutex and unwrap to get the PgConnection
    {
        Ok(_) => {
            HttpResponse::Ok().body("User registered successfully")
        }
        Err(_) => {
            HttpResponse::BadRequest().body("Error inserting user into database")
        }
    }
}

pub async fn login_user(user_data: Json<LoginUser>, pool:Data<DbPool>) -> HttpResponse {
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
    session: Json<Session>,
    redis: Data<Client>,
    pool: Data<DbPool>,
) -> HttpResponse {
    // Extract JWT token from request headers
    let token_validation = validate_token(req, "server".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            match redis.get_ref().get_multiplexed_async_connection().await {
                Ok(mut con) => {
                    // Store session data in Redis using the session id as the key
                    let session_id: Uuid = Uuid::new_v4();
                    let _: Result<(), RedisError> = con
                        .set(session_id.to_string(), serde_json::to_string(&session.into_inner()).unwrap())
                        .await;
                    //Add the session to a table in the database
                    let mut conn = pool.get().expect("Couldn't get db connection from pool");

                    // Insert new user into the database with the session id
                    diesel::insert_into(sessions::table)
                        .values(sessions::id.eq(session_id))
                        .execute(&mut conn) // Lock the Mutex and unwrap to get the PgConnection
                        .expect("Error inserting user into database");

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
    redis: Data<Client>,
    pool: Data<DbPool>,
    other_username: web::Path<String>,
) -> HttpResponse {
    // Extract JWT token from request headers
    let token_validation = validate_token(req, "client".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            //Get all non-empty the sessions from the database
            let mut conn = pool.get().expect("Couldn't get db connection from pool");
            let sessions: Vec<DBSession> = sessions::table
                .filter(sessions::is_empty.eq(false))
                .load(&mut conn)
                .expect("Error loading sessions");

            let session_id: Uuid;
            if (sessions.len() == 0) {
                //Get the first empty session from the database
                let empty_session: Uuid = match sessions::table
                    .select(sessions::id)
                    .filter(sessions::is_empty.eq(true))
                    .first(&mut conn)
                {
                    Ok(id) => id,
                    Err(_) => {
                        return HttpResponse::BadRequest().body("No empty sessions available");
                    }
                };
                session_id = empty_session;
            } else {
                //Get the user data from the database
                let user_data: User  = match users::table
                    .filter(users::username.eq(other_username.into_inner()))
                    .first(&mut conn)
                {
                    Ok(id) => id,
                    Err(_) => {
                        return HttpResponse::BadRequest().body("Invalid username");
                    }
                };

                //Get the rank of the user
                let rank: Rank = match ranks::table
                    .filter(ranks::id.eq(&user_data.rank_id))
                    .first(&mut conn)
                {
                    Ok(id) => id,
                    Err(_) => {
                        return HttpResponse::BadRequest().body("Invalid rank id");
                    }
                };

                //Get the average kda of the user
                let kda: f32 = user_data.kda;

                //Get the session with the closest average kda to the user
                let mut closest_session: DBSession = sessions[0].clone();
                let mut closest_distance: f32 = (kda - sessions[0].average_kda).abs();
                for session in sessions.iter() {
                    let distance = (kda - session.average_kda).abs();
                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_session = session.clone();
                    }
                }
                session_id = closest_session.id;
            }


            // Get the session from Redis
            match redis.get_ref().get_multiplexed_async_connection().await {
                Ok(mut con) => {
                    // Retrieve session data from Redis
                    let session_data: Result<String, RedisError> = con.get(session_id.to_string()).await;
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

//Connect the  player to a session by adding his id to the session
pub async fn connect_to_session(
    req: HttpRequest,
    pool: Data<DbPool>,
    redis: Data<Client>,
    session_id: web::Path<Uuid>,
    other_username: web::Path<String>,
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "client".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

            let user_id: Uuid = match users::table
                .select(users::id)
                .filter(users::username.eq(other_username.into_inner()))
                .first(&mut conn)
            {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::BadRequest().body("Invalid username");
                }
            };

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
                Err(_) => HttpResponse::InternalServerError().body("Failed to connect to Redis")
            }
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

pub async fn validate_achievement(user_data: Json<AchievementValidation>, pool:Data<DbPool>, http_request: HttpRequest) -> HttpResponse {
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

            match diesel::insert_into(user_achievements::table)
                .values(&user_achievement)
                .execute(&mut conn)
            {
                Ok(_) => {
                    HttpResponse::Ok().body("Achievement validated successfully")
                }
                Err(_) => {
                    return HttpResponse::BadRequest().body("Error inserting user achievement into database");
                }
            }
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }

}

//Get an achievement from the database by id (id in the URL)
pub async fn get_achievement_by_id(
    req: HttpRequest,
    pool: Data<DbPool>,
    achievement_id: web::Path<Uuid>,
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "client".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

            // Retrieve achievement from database
            let achievement: Achievement = match achievements::table
                .filter(achievements::id.eq(achievement_id.into_inner()))
                .first(&mut conn)
            {
                Ok(achievement) => achievement,
                Err(_) => {
                    return HttpResponse::NotFound().body("Achievement not found");
                }
            };

            HttpResponse::Ok().json(achievement)
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

//Get all the achievements from the database
pub async fn get_all_achievements(
    req: HttpRequest,
    pool: Data<DbPool>,
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "all".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

            // Retrieve all achievements from database
            let achievements: Vec<Achievement> = achievements::table
                .load(&mut conn)
                .expect("Error loading achievements");

            HttpResponse::Ok().json(achievements)
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

//Get all the validate achievements from the database by user id
pub async fn get_user_achievements(
    req: HttpRequest,
    pool: Data<DbPool>,
    username_into: web::Path<String>,
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "client".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

            let user_id: Uuid = match users::table
                .select(users::id)
                .filter(users::username.eq(username_into.into_inner()))
                .first(&mut conn)
            {
                Ok(id) => id,
                Err(_) => {
                    return HttpResponse::BadRequest().body("Invalid username");
                }
            };

            // Retrieve all achievements from database
            let achievements: Vec<Achievement> = user_achievements::table
                .inner_join(achievements::table)
                .select(achievements::all_columns)
                .filter(user_achievements::user_id.eq(&user_id))
                .load(&mut conn)
                .expect("Error loading achievements");

            HttpResponse::Ok().json(achievements)
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

//update kda of a user by user id
pub async fn update_kda(
    req: HttpRequest,
    pool: Data<DbPool>,
    user_data: Json<KdaUpdate>
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "server".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            let user_data = user_data.into_inner();
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

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

            // Update user KDA
             match diesel::update(users::table.filter(users::id.eq(user_id)))
                .set(users::kda.eq(user_data.new_kda))
                .execute(&mut conn)
             {
                Ok(_) => {
                    HttpResponse::Ok().body("KDA updated successfully")
                }
                Err(_) => {
                    return HttpResponse::BadRequest().body("Error updating user KDA");
                }
             }
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

//Get kda of a user by username
pub async fn get_kda(
    req: HttpRequest,
    pool: Data<DbPool>,
    username_into: web::Path<String>,
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "all".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

            // Retrieve user from database
            let user: User = match users::table
                .filter(users::username.eq(&username_into.into_inner()))
                .first(&mut conn)
            {
                Ok(user) => user,
                Err(_) => {
                    return HttpResponse::NotFound().body("User not found");
                }
            };

            HttpResponse::Ok().json(user.kda)
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

//Get all ranks from the database
pub async fn get_all_ranks(
    req: HttpRequest,
    pool: Data<DbPool>,
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "all".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

            // Retrieve all ranks from database
            let ranks: Vec<Rank> = ranks::table
                .load(&mut conn)
                .expect("Error loading ranks");

            HttpResponse::Ok().json(ranks)
        }
        1 => HttpResponse::Unauthorized().body("Unauthorized"),
        2 => HttpResponse::Forbidden().body("Permission denied"),
        _ => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

//Update user rank by user id
pub async fn update_rank(
    req: HttpRequest,
    pool: Data<DbPool>,
    user_data: Json<RankUpdate>
) -> HttpResponse {
    //Validate the JWT token
    let token_validation = validate_token(req, "server".to_string());
    //Switch on the token validation result
    match token_validation {
        0 => {
            let user_data = user_data.into_inner();
            // Establish a database connection
            let mut conn = pool.get().expect("Couldn't get db connection from pool");

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

            // Update user rank
            match diesel::update(users::table.filter(users::id.eq(user_id)))
                .set(users::rank_id.eq(user_data.new_rank_id))
                .execute(&mut conn)
            {
                Ok(_) => {
                    HttpResponse::Ok().body("Rank updated successfully")
                }
                Err(_) => {
                    return HttpResponse::BadRequest().body("Error updating user rank");
                }
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
            if claims.claims.role == role_value || role_value == "all"{
                0
            } else {
                2
            }
        }
        Err(_) => 1,
    }
}
