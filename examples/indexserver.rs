extern crate hprtree;

use std::sync::Mutex;

use actix_web::{
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use hprtree::{BBox, HPRTree, HPRTreeBuilder, Point};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Element {
    pub x: f32,
    pub y: f32,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindBBox {
    pub minx: f32,
    pub miny: f32,
    pub maxx: f32,
    pub maxy: f32,
}

impl Into<BBox> for FindBBox {
    fn into(self) -> BBox {
        BBox {
            minx: self.minx,
            miny: self.miny,
            maxx: self.maxx,
            maxy: self.maxy,
        }
    }
}

#[post("/add")]
async fn add(
    index: Data<Mutex<HPRTree<Element>>>,
    index_builder: Data<Mutex<HPRTreeBuilder<Element>>>,
    data: web::Json<Element>,
) -> impl Responder {
    let mut index = index.lock().unwrap();
    let mut index_builder = index_builder.lock().unwrap();
    let data = data.into_inner();
    let pt = Point {
        x: data.x,
        y: data.y,
    };
    index_builder.insert(data, pt);
    index_builder.sort_items();
    *index = index_builder.clone().build_sorted();

    HttpResponse::Ok()
}

#[post("/find")]
async fn find(index: Data<Mutex<HPRTree<Element>>>, data: web::Json<FindBBox>) -> impl Responder {
    let index = index.lock().unwrap();
    let data: BBox = data.into_inner().into();

    let res = index.query(&data);

    HttpResponse::Ok().json(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let builder = HPRTreeBuilder::<Element>::new(16);
    let index_builder = Data::new(Mutex::new(builder.clone()));
    let index = Data::new(Mutex::new(builder.build()));

    HttpServer::new(move || {
        App::new()
            .app_data(index_builder.clone())
            .app_data(index.clone())
            .service(add)
            .service(find)
    })
    .bind(("localhost", 3030))?
    .run()
    .await
}
