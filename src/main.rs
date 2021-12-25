use actix_web::{get, post, web::{self, Form}, App, HttpResponse, HttpServer, Responder, HttpRequest, http::header::ContentType};
use std::time::Instant;

use serde::{Deserialize};

use image::io::Reader as ImageReader;

#[derive(Deserialize)]
struct ImageUrl {
    url: String
}

#[post("/embed")]
// We fetch the image ourself so that we don't risk accidentally revealing our users IP
async fn embed_image(query: web::Json<ImageUrl>) -> impl Responder {
    let tgt_url = query.url.clone();
    let now = Instant::now();
    let test = tokio::spawn(async move {
        reqwest::get(tgt_url)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
    }).await.unwrap();

    println!("Elapsed: {:.2?}", now.elapsed());
    HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(test)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(embed_image)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

