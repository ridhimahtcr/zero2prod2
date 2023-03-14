//! src/startup.rs
use crate::routes::{subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::{ Pool, Postgres};
use std::net::TcpListener;
use crate::health_check;
use actix_web::web::Data;
use sqlx::pool::PoolOptions;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener, db_pool: Pool<Postgres>
) -> Result<Server, std::io::Error> {
    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
// Middlewares are added using the `wrap` method on `App`
        .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}
