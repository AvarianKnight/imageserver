use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{
    error::ErrorBadRequest, get, http::header::ContentType, post, web, App, Error, FromRequest,
    HttpRequest, HttpResponse, HttpServer,
};

use std::io::Write;
use std::{fs, io::Read};

use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Url {
    url: String
}

#[get("/external")]
// We fetch the image ourself so that we don't risk accidentally revealing our users IP
async fn external_image(
    url: web::Query<Url>
) -> Result<HttpResponse, Error> {
    let res = reqwest::get(&*url.url).await.unwrap();

    // Early return if the status isn't a success, usually means that the target website doesn't exist
    if !res.status().is_success() {
        return Err(ErrorBadRequest(format!(
            "Target website returned status code {}.",
            res.status()
        )));
    }

    let data = res.bytes().await.unwrap().to_vec();

    if !infer::is_image(&data) {
        return Err(ErrorBadRequest(
            "The target website didn't return an image.",
        ));
    }

    let mut builder = HttpResponse::Ok();
    builder.insert_header(CacheControl(vec![CacheDirective::MaxAge(86400u32)]));
    builder.content_type(ContentType::png());

    Ok(builder.body(data))
}

#[derive(Serialize)]
struct ImageStruct {
    link: String,
}

#[derive(Serialize)]
struct ReturnData {
    data: ImageStruct,
}

#[post("/image")]
async fn upload_image(req: HttpRequest, config: web::Data<Config>) -> Result<HttpResponse, Error> {
    let data = web::Bytes::extract(&req).await?.to_vec();

    if !infer::is_image(&data) {
        return Err(ErrorBadRequest("The provided data wasn't an image."));
    }

    let unique_signature = Uuid::new_v4();
    let kind = infer::get(&data).unwrap();
    let image_url = format!("{}.{}", unique_signature, kind.extension());

    let mut file = fs::File::create(format!("./images/{}", image_url)).unwrap();

    file.write_all(&data).unwrap();

    let return_data = serde_json::to_string(&ReturnData {
        data: ImageStruct {
            link: format!("{}/get/{}", config.domain, image_url),
        },
    })
    .unwrap();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(return_data))
}

#[get("/fetch/{image}")]
async fn fetch_image(image_name: web::Path<String>) -> Result<HttpResponse, Error> {
    let image = fs::read(format!("./images/{}", image_name))?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::png())
        .body(image))
}

#[derive(Deserialize, Clone)]
struct Config {
    ip: String,
    domain: String,
    port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    match fs::create_dir("./images") {
        Ok(ok) => ok,
        Err(err) => {
            // We don't want to completely error out just because the file already exists, this would be expected behaviour.
            if err.kind() != std::io::ErrorKind::AlreadyExists {
                panic!(
                    "Failed to create image directory with the following error: {}",
                    err
                )
            }
        }
    };

    let mut config_file = String::new();
    match fs::File::open("./config.toml") {
        Ok(mut file) => {
            file.read_to_string(&mut config_file).unwrap();
        }
        Err(err) => panic!(
            "Failed to read the config.toml with the following error: {}",
            err
        ),
    }

    let config: Config = toml::from_str(config_file.as_str()).unwrap();
    let config_clone = web::Data::new(config.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(config_clone.clone())
            .service(external_image)
            .service(upload_image)
            .service(fetch_image)
    })
    .bind((config.ip, config.port))?
    .run()
    .await
}
