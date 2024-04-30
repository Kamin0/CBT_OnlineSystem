// main.rs
use actix_web::{web, App, HttpServer};
use std::{env, io};
use actix_web::web::Data;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;

mod handlers;
mod models;
mod schema;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    let redis_data = web::Data::new(client);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(redis_data.clone())
            .route("/hello", web::get().to(handlers::hello))
            .route("/register", web::post().to(handlers::register_user))
            .route("/login", web::post().to(handlers::login_user))
            .route("/session/{other_username}", web::get().to(handlers::request_session))
            .route("/session", web::post().to(handlers::register_session))
            .route("/connect", web::post().to(handlers::connect_to_session))
            .route("/achievement", web::post().to(handlers::validate_achievement))
            .route("/achievement/{achievement_id}", web::get().to(handlers::get_achievement_by_id))
            .route("/achievements", web::get().to(handlers::get_all_achievements))
            .route("/user_achievements/{username_into}", web::get().to(handlers::get_user_achievements))
            .route("/kda", web::put().to(handlers::update_kda))
            .route("/kda/{username_into}", web::get().to(handlers::get_kda))
            .route("/ranks", web::get().to(handlers::get_all_ranks))
            .route("/rank", web::put().to(handlers::update_rank))
            .route("/nb_games/{username_into}", web::put().to(handlers::update_games_played))
            .route("/nb_games/{username_into}", web::get().to(handlers::get_games_played))
            .route("/get_ip",web::get().to(handlers::get_ip))
    })
        .bind("0.0.0.0:8000")?
        .run()
        .await
}

