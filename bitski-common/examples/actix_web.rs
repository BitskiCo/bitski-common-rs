use bitski_common::{
    actix_web::{web, HttpResponse, HttpServer},
    actix_web_app,
    env::parse_env_addr_or_default,
    with_instruments,
};

#[with_instruments]
#[tokio::main]
async fn main() {
    let addr = parse_env_addr_or_default().unwrap();

    HttpServer::new(move || actix_web_app!().route("/", web::get().to(livez)))
        .workers(1)
        .bind(addr)
        .unwrap()
        .run()
        .await
        .unwrap();
}

async fn livez() -> HttpResponse {
    HttpResponse::Ok().finish()
}
