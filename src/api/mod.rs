use actix_web::{App, HttpServer, middleware, web};
use crate::api::health::health;

mod health;

pub async fn serve(url: &str) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new().wrap(middleware::Logger::default()).service(web::scope("/health").route("", web::get().to(health)))
    }).bind((url, 8080))?
        .run()
        .await
}
