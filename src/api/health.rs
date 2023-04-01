use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, PartialEq, Debug, Deserialize)]
struct Response {
    status: String,
}

pub async fn health() -> impl Responder {
    HttpResponse::Ok().content_type("application/json").body(
        serde_json::to_string(&Response {
            status: "OK".parse().unwrap(),
        })
        .unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use crate::api::health::{health, Response};
    use actix_web::body::to_bytes;
    use actix_web::http::{header, StatusCode};
    use actix_web::{http::header::ContentType, test, web, App};

    #[actix_web::test]
    async fn test_health_route_ok() {
        let mut app = test::init_service(App::new().route("/", web::get().to(health))).await;
        let res = test::TestRequest::get()
            .uri("/")
            .send_request(&mut app)
            .await;
        assert!(res.status().is_success());
        let result: Response = test::read_body_json(res).await;
        assert_eq!(
            result,
            (Response {
                status: "OK".parse().unwrap()
            })
        );
    }
}
