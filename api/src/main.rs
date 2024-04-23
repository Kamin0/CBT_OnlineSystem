// main.rs
use actix_web::{web, App, HttpServer};
use std::env;
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

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .route("/hello", web::get().to(handlers::hello))
            .route("/register", web::post().to(handlers::register_user))
            .route("/login", web::post().to(handlers::login_user))
    })
        .bind("127.0.0.1:8000")?
        .run()
        .await
}

