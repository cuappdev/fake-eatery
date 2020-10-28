use std::{fs, io, path::Path};

use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Clone)]
struct ExpandedEatery {
    id: usize,
    name: String,
    category: Vec<String>,
    open_time: String,
    close_time: String,
    rating: f32,
    photo: String,
    address: String,
    phone_number: String,
    reviews: Vec<String>,
}

impl ExpandedEatery {
    fn get_all<P: AsRef<Path>>(path: P) -> Result<Vec<Self>, io::Error> {
        Ok(fs::read_dir(path)?
            .map(|f| {
                let mut contents = String::new();
                File::open(f?.path())?.read_to_string(&mut contents)?;
                serde_json::from_str::<ExpandedEatery>(&contents)//.unwrap_or(Err(io::Error::new(io::ErrorKind::InvalidData, "bad json")))
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad json"))
            })
            .filter_map(|e| e.ok())
            .collect())
    }
}

#[derive(Serialize)]
struct BasicEatery {
    id: usize,
    name: String,
    category: Vec<String>,
    open_time: String,
    close_time: String,
    rating: f32,
    photo: String,
    address: String,
    phone_number: String,
}

impl From<ExpandedEatery> for BasicEatery {
    fn from(e: ExpandedEatery) -> Self {
        BasicEatery {
            id: e.id,
            name: e.name,
            category: e.category,
            open_time: e.open_time,
            close_time: e.close_time,
            rating: e.rating,
            photo: e.photo,
            address: e.address,
            phone_number: e.phone_number,
        }
    }
}

#[derive(Serialize)]
struct EateryList {
    restaurants: Vec<BasicEatery>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/eateries/")]
async fn eateries(eateries: web::Data<Vec<ExpandedEatery>>) -> impl Responder {
    let ret = eateries
        .get_ref()
        .iter()
        .cloned()
        .map(BasicEatery::from)
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(EateryList { restaurants: ret })
}

#[get("/eateries/{id}/")]
async fn eatery(
    web::Path(id): web::Path<usize>,
    eateries_vec: web::Data<Vec<ExpandedEatery>>,
) -> impl Responder {
    let ret = eateries_vec.get_ref().iter().find(|e| e.id == id);

    match ret {
        Some(e) => HttpResponse::Ok().json2(e),
        None => HttpResponse::NotFound().body("No eatery with that id"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let eateries_vec = ExpandedEatery::get_all("./eateries").unwrap();

        App::new()
            .wrap(middleware::NormalizePath::default())
            .data(eateries_vec)
            .service(hello)
            .service(echo)
            .service(eateries)
            .service(eatery)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
