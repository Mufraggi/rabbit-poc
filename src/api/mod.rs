use crate::api::health::health;
use actix_web::dev::Server;
use actix_web::{middleware, web, App, HttpServer};

mod health;

pub fn serve(url: &str) -> Server {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::scope("/health").route("", web::get().to(health)))
    })
    .bind((url, 8080))
    .unwrap()
    .run()
}
