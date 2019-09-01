use futures::{Future, IntoFuture};
use log::info;
use std::str;

use actix_web::{
    get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web::web::route;
use bytes::Bytes;

#[get("/resource1/{name}/index.html")]
fn index(req: HttpRequest, name: web::Path<String>) -> String {
    println!("REQ: {:?}", req);
    format!("Hello: {}!\r\n", name)
}

fn index_async(req: HttpRequest) -> impl IntoFuture<Item = &'static str, Error = Error> {
    println!("REQ: {:?}", req);
    Ok("Hello world!\r\n")
}

#[get("/")]
fn no_params() -> &'static str {
    "Hello world!\r\n"
}

fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(index)
            .service(no_params)
            .service(
                web::resource("/resource2/index.html")
                    .wrap(
                        middleware::DefaultHeaders::new().header("X-Version-R2", "0.3"),
                    )
                    .default_service(
                        web::route().to(|| HttpResponse::MethodNotAllowed()),
                    )
                    .route(web::get().to_async(index_async)),
            )
            .service(web::resource("/test1.html").to(|| "Test\r\n"))
            .service( catfacts_async)
            .service( catfacts_async2)
    })
        .bind("127.0.0.1:8080")?
        .workers(1)
        .run()
}


#[get("/catfacts")]
fn catfacts_async(req: HttpRequest) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    awc::Client::new()
        .get("https://cat-fact.herokuapp.com/facts/random") // <- Create request builder
        .header("User-Agent", "Actix-web")
        .send() // <- Send http request
        .from_err()
        .and_then(|mut response| {
            // <- server http response
            info!("Response: {:?}", response);

            // read response body
            response
                .body()
                .from_err()
                .map(|body| {
                    // Do something with body here
                    // Maybe pass handler Fn in?
                    info!("Downloaded: {:?} bytes", body.len());
                    HttpResponse::Ok().content_type("application/json").body(body)
                })

        })
}

#[get("/catfacts2")]
fn catfacts_async2(req: HttpRequest) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    async_req("https://cat-fact.herokuapp.com/facts/random")
        .map(|body| {
            // Do something with body here
            info!("Downloaded: {:?} bytes", body.len());
            HttpResponse::Ok().content_type("application/json").body(body)
        })
}

fn async_req(target: &str) -> impl Future<Item = Bytes, Error = actix_web::Error>{
    awc::Client::new()
        .get(target) // <- Create request builder
        .header("User-Agent", "Actix-web")
        .send() // <- Send http request
        .from_err()
        .and_then(|mut response| {
            // <- server http response
            info!("Response: {:?}", response);

            // read response body
            response
                .body()
                .from_err()
        })
}
