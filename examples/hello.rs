use chrono::Utc;
use serde::Deserialize;
use tokio;
use warp::{
    fs::File,
    http::HeaderValue,
    hyper::{self, Body, HeaderMap, Response},
    Filter, Reply,
};

#[derive(Deserialize)]
struct GetIcon {
    ext: String,
    size: i32,
}

#[cfg(target_os = "linux")]
fn init() {
    gtk::init().unwrap();
}
#[cfg(target_os = "windows")]
fn init() {}
#[cfg(target_os = "macos")]
fn init() {}

#[tokio::main]
async fn main() {
    init();

    async fn get_icon(param: GetIcon) -> Result<impl warp::Reply, warp::Rejection> {
        let bytes = systemicons::get_icon(&param.ext, param.size).unwrap();
        let body = hyper::Body::from(bytes);
        let mut response = Response::new(body);
        let headers = response.headers_mut();
        let mut header_map = create_headers();
        header_map.insert("Content-Type", HeaderValue::from_str("image/png").unwrap());
        headers.extend(header_map);
        Ok(response)
    }

    let route_get_icon = warp::path("geticon")
        .and(warp::path::end())
        .and(warp::query::query())
        .and_then(get_icon);

    fn add_headers(reply: File) -> Response<Body> {
        let mut header_map = HeaderMap::new();
        let now = Utc::now();
        let now_str = now.format("%a, %d %h %Y %T GMT").to_string();
        header_map.insert("Expires", HeaderValue::from_str(now_str.as_str()).unwrap());
        header_map.insert("Server", HeaderValue::from_str("My Server").unwrap());

        let mut res = reply.into_response();
        let headers = res.headers_mut();
        headers.extend(header_map);
        res
    }

    let route = warp::fs::dir(".").map(add_headers);

    let routes = route.or(route_get_icon);

    let port = 8888;
    println!("Serving example on http://localhost:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

fn create_headers() -> HeaderMap {
    let mut header_map = HeaderMap::new();
    let now = Utc::now();
    let now_str = now.format("%a, %d %h %Y %T GMT").to_string();
    header_map.insert("Expires", HeaderValue::from_str(now_str.as_str()).unwrap());
    header_map.insert("Server", HeaderValue::from_str("My Server").unwrap());
    header_map
}
