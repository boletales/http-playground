use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
struct Count {
    value: i32,
}

type SharedCount = Arc<Mutex<Count>>;

async fn handle_request(req: Request<Body>, count: SharedCount) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();
    match (req.method(), path) {
        // クライアントページ
        (&Method::GET, "/") => {
            let html = include_str!("../static/index.html");
            Ok(Response::new(Body::from(html)))
        }
        // カウント取得
        (&Method::GET, "/getcount") => {
            let count = count.lock().unwrap().clone();
            let body = serde_json::to_string(&count).unwrap();
            Ok(Response::new(Body::from(body)))
        }
        // カウント加算
        (&Method::POST, "/addcount") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let add: Count = serde_json::from_slice(&whole_body).unwrap_or(Count { value: 1 });
            let mut count = count.lock().unwrap();
            count.value += add.value;
            let body = serde_json::to_string(&*count).unwrap();
            Ok(Response::new(Body::from(body)))
        }
        // カウントリセット
        (&Method::PUT, "/resetcount") => {
            let mut count = count.lock().unwrap();
            count.value = 0;
            let body = serde_json::to_string(&*count).unwrap();
            Ok(Response::new(Body::from(body)))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
async fn main() {
    let count = Arc::new(Mutex::new(Count { value: 0 }));
    let make_svc = make_service_fn(move |_| {
        let count = count.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_request(req, count.clone())
            }))
        }
    });
    let addr = ([127, 0, 0, 1], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);
    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
