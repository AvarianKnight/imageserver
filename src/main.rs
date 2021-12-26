use actix_web::{get, post, App, web, HttpResponse, HttpServer, http::header::ContentType, Error};
use std::fs;
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, Instant}; 

use serde::{Deserialize};

#[derive(Deserialize)]
struct ImageUrl {
    url: String
}

struct ImageData {
    last_access: SystemTime,
    image_data: Vec<u8>
}

impl ImageData {
    pub fn new(image: Vec<u8>) -> Self {
        Self {
            image_data: image,
            last_access: SystemTime::now()
        }
    }

    fn update_last_access(&mut self) {
        self.last_access = SystemTime::now()
    }
}

#[post("/embed")]
// We fetch the image ourself so that we don't risk accidentally revealing our users IP
async fn embed_image(query: web::Json<ImageUrl>) -> Result<HttpResponse, Error> {
    let tgt_url = query.url.clone();
    let data = reqwest::get(tgt_url)
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(data))
}

#[get("/{image}")]
async fn fetch_image(image_name: web::Path<String>, data: web::Data<Mutex<HashMap<String, ImageData>>>) -> Result<HttpResponse, Error> {
    let time = Instant::now();
    let mut data = data.lock().unwrap();
    let image_data = data.get_mut(&*image_name);

    if image_data.is_some() {
        println!("We already have that image!");
        let hash_data = image_data.unwrap();
        let image = hash_data.image_data.clone();

        // hash_data.update_last_access();

        println!("Took {:0.2?} to complete the request", time.elapsed());

        Ok(HttpResponse::Ok()
            .content_type(ContentType::png())
            .body(image))
    } else {
        println!("We don't have that image in the cache, fetching and caching.");
        let image = fs::read(format!("./images/{}", image_name))?;
        data.insert(image_name.clone(), ImageData::new(image.clone()));

        println!("Took {:0.2?} to complete the request", time.elapsed());
        Ok(HttpResponse::Ok()
            .content_type(ContentType::png())
            .body(image))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match fs::create_dir("./images") {
        Ok(ok) => ok,
        Err(err) => {
            // We don't want to completely error out just because the file already exists
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                panic!("Failed to create image directory with the following reason: {}", err)
            }
        }
    };

    // Cache image data so we don't have to do constant file reads
    // These caches should be cleared 12hrs after last access

    let data: web::Data<Mutex<HashMap<String, ImageData>>> = web::Data::new(Mutex::new(HashMap::new()));

    let moveable_data = data.clone();

    HttpServer::new(move || {
        App::new()
            .app_data(moveable_data.clone())
            .service(embed_image)
            .service(fetch_image)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}