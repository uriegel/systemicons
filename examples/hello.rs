use chrono::Utc;
use serde::{Deserialize};
use tokio;
use tokio_util::codec::{BytesCodec, FramedRead};
use warp::{Filter, Reply, fs::File, http::HeaderValue, hyper::{self, Body, HeaderMap, Response, StatusCode}};

static NOTFOUND: &[u8] = b"Not Found";

#[derive(Deserialize)]
struct GetIcon {
    ext: String,
}

const SIZE: i32 = 16;

#[tokio::main]
async fn main() {

    gtk::init().unwrap();

    async fn get_icon(param: GetIcon) -> Result<impl warp::Reply, warp::Rejection> {
        let bytes = systemicons::get_icon(&param.ext, SIZE);
        let body = hyper::Body::from(bytes);
        let mut response = Response::new(body);
        let headers = response.headers_mut();
        let mut header_map = create_headers();
        header_map.insert("Content-Type", HeaderValue::from_str("image/png").unwrap());
        headers.extend(header_map);
        Ok (response)        
    }

    async fn get_icon_as_file(param: GetIcon) -> Result<impl warp::Reply, warp::Rejection> {
        let path = systemicons::get_icon_as_file(&param.ext, SIZE);

        match tokio::fs::File::open(path.clone()).await {
            Ok(file) => {
                let stream = FramedRead::new(file, BytesCodec::new());
                let body = hyper::Body::wrap_stream(stream);
                let mut response = Response::new(body);

                let headers = response.headers_mut();
                let mut header_map = create_headers();
                if let Some(ext_index) = path.rfind('.') {
                    let ext = &path[ext_index..].to_lowercase();
                    let content_type = match ext.as_str() {
                        ".png" => "image/png".to_string(),
                        ".svg" => "image/svg".to_string(),
                        _ => "image/jpg".to_string(),
                    };
                    header_map.insert(
                        "Content-Type",
                        HeaderValue::from_str(&content_type).unwrap(),
                    );
                }
                headers.extend(header_map);
                Ok(response)
            }
            Err(err) => {
                println!("Could not get icon: {}", err);
                Ok(not_found())
            }
        }
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

    let routes = route
        .or(route_get_icon);

    let port = 8888;
    println!("Serving example on http://localhost:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

fn create_headers() -> HeaderMap {
    let mut header_map = HeaderMap::new();
    let now = Utc::now();
    let now_str = now.format("%a, %d %h %Y %T GMT").to_string();
    header_map.insert("Expires", HeaderValue::from_str(now_str.as_str()).unwrap());
    header_map.insert("Server", HeaderValue::from_str("Mein Server").unwrap());
    header_map
}


fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(NOTFOUND.into())
        .unwrap()
}

// TODO 64px .pdf 2ms 39.2 kB as file
// TODO 64px .pdf 2ms 39.1 kB as buf