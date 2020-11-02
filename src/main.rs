use std::{fs, io, path::Path};

use actix_web::{get, middleware, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

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
                let contents = fs::read_to_string(f?.path())?;
                serde_json::from_str::<ExpandedEatery>(&contents)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad json"))
            })
            .filter_map(Result::ok)
            .collect())
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
struct EateryReq {
    id: usize,
}

#[post("/eatery/")]
async fn eatery(
    web::Json(EateryReq { id }): web::Json<EateryReq>,
    eateries_vec: web::Data<Vec<ExpandedEatery>>,
) -> impl Responder {
    let ret = eateries_vec.get_ref().iter().find(|e| e.id == id);

    match ret {
        Some(e) => HttpResponse::Ok().json2(e),
        None => HttpResponse::NotFound().body("No eatery with that id"),
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    name: String,
}

#[get("/eateries/search/")]
async fn search(
    web::Query(SearchQuery { name }): web::Query<SearchQuery>,
    eateries_vec: web::Data<Vec<ExpandedEatery>>,
) -> impl Responder {
    let s = name.to_lowercase();

    let results = eateries_vec
        .get_ref()
        .iter()
        .filter(|e| e.name.to_lowercase().contains(&s))
        .cloned()
        .map(BasicEatery::from)
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(EateryList {
        restaurants: results.clone(),
    })
}

fn eatery_config(cfg: &mut web::ServiceConfig) {
    let mut eateries_vec = ExpandedEatery::get_all("./eateries").unwrap();
    eateries_vec.sort_by_key(|e| e.id);

    cfg.data(eateries_vec)
        .service(eateries)
        .service(eatery)
        .service(search);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath::default())
            .service(hello)
            .service(echo)
            .configure(eatery_config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use crate::{eatery_config, EateryList, EateryReq, ExpandedEatery};
    use actix_web::{middleware, test, App};

    #[actix_rt::test]
    async fn test_eateries() {
        let mut app = test::init_service(
            App::new()
                .wrap(middleware::NormalizePath::default())
                .configure(eatery_config),
        )
        .await;
        let req = test::TestRequest::get().uri("/eateries").to_request();
        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
        let l: EateryList = test::read_body_json(res).await;
        assert_eq!(l.restaurants.len(), 10);
    }

    #[actix_rt::test]
    async fn test_eatery() {
        let mut app = test::init_service(
            App::new()
                .wrap(middleware::NormalizePath::default())
                .configure(eatery_config),
        )
        .await;

        for id in 0..10 {
            let req = test::TestRequest::post()
                .uri("/eatery")
                .set_json(&EateryReq { id })
                .to_request();
            let res = test::call_service(&mut app, req).await;
            assert!(res.status().is_success());
            let e: ExpandedEatery = test::read_body_json(res).await;
            assert_eq!(e.id, id);
        }
    }

    #[actix_rt::test]
    async fn test_together() {
        let mut app = test::init_service(
            App::new()
                .wrap(middleware::NormalizePath::default())
                .configure(eatery_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/eateries").to_request();
        let res = test::call_service(&mut app, req).await;
        assert!(res.status().is_success());
        let l: EateryList = test::read_body_json(res).await;
        assert_eq!(l.restaurants.len(), 10);

        for be in l.restaurants {
            let req = test::TestRequest::post()
                .uri("/eatery")
                .set_json(&EateryReq { id: be.id })
                .to_request();
            let res = test::call_service(&mut app, req).await;
            assert!(res.status().is_success());
            let e: ExpandedEatery = test::read_body_json(res).await;
            assert_eq!(e.id, be.id);
            assert_eq!(e.phone_number, be.phone_number);
            assert_eq!(e.address, be.address);
            assert_eq!(e.photo, be.photo);
            assert_eq!(e.open_time, be.open_time);
            assert_eq!(e.close_time, be.close_time);
            assert_eq!(e.rating, be.rating);
            assert_eq!(e.category, be.category);
            assert_eq!(e.name, be.name);
        }
    }

    #[actix_rt::test]
    async fn test_search() {
        let mut app = test::init_service(
            App::new()
                .wrap(middleware::NormalizePath::default())
                .configure(eatery_config),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/eateries/search/?name=c")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        let l: EateryList = test::read_body_json(res).await;
        assert_eq!(l.restaurants.len(), 4);

        let req = test::TestRequest::get()
            .uri("/eateries/search/?name=xyz")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        let l: EateryList = test::read_body_json(res).await;
        assert_eq!(l.restaurants.len(), 0);

        let req = test::TestRequest::get()
            .uri("/eateries/search/?name=college")
            .to_request();

        let res = test::call_service(&mut app, req).await;
        let l: EateryList = test::read_body_json(res).await;
        assert_eq!(l.restaurants.len(), 2);
    }
}
