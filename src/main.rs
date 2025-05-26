use serde::{Deserialize, Serialize};
use std::{
    convert::Infallible,
    sync::{Arc, Mutex},
};
use hyper::{
    server::conn::http1,
    service::service_fn,
    body::{Bytes, Incoming},
    {Request, Response, Method, StatusCode},
};
use hyper_util::rt::TokioIo;
use http_body_util::{BodyExt, Full};
use tokio::net::TcpListener;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Clone)]
struct Count {
    value: i32,
}

type SharedCount = Arc<Mutex<Count>>;

async fn handle_request(req: Request<Incoming>, count: SharedCount) -> Result<Response<Full<Bytes>>, Infallible> {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    println!("\n\n\nRequest Received!");
    println!("* Request: {} {}", method, path);
    println!("* Header:");
    for (k, v) in req.headers() {
        println!("    * {}: {:?}", k, v);
    }
    let whole_body = req.into_body().collect().await.unwrap().to_bytes();
    println!("* Body: {:?}", String::from_utf8_lossy(&whole_body));
    
    let res = match (&method, path.as_str()) {
        // クライアントページ
        (&Method::GET, "/") => {
            let html = include_str!("../static/index.html");
            let mut res = Response::new(Full::new(Bytes::from(html)));
            res.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("text/html"),
            );
            res
        }
        // カウント取得
        (&Method::GET, "/getcount") => {
            let count = count.lock().unwrap().clone();
            let body = serde_json::to_string(&count).unwrap();
            let mut res = Response::new(Full::new(Bytes::from(body)));
            res.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            );
            res
        }
        // カウント加算
        (&Method::POST, "/addcount") => {
            match serde_json::from_slice::<Count>(&whole_body) {
                Ok(add) => {
                    let mut count = count.lock().unwrap();
                    count.value += add.value;
                    let body = serde_json::to_string(&*count).unwrap();
                    let mut res = Response::new(Full::new(Bytes::from(body)));
                    res.headers_mut().insert(
                        hyper::header::CONTENT_TYPE,
                        hyper::header::HeaderValue::from_static("application/json"),
                    );
                    res
                },
                Err(_) => {
                    let mut res: Response<Full<Bytes>> = Response::new(Full::new(Bytes::from("Bad Request (Invalid JSON)")));
                    *res.status_mut() = StatusCode::BAD_REQUEST;
                    res.headers_mut().insert(
                        hyper::header::CONTENT_TYPE,
                        hyper::header::HeaderValue::from_static("text/plain"),
                    );
                    res
                }
            }
        }
        // カウントリセット
        (&Method::POST, "/resetcount") => {
            let mut count = count.lock().unwrap();
            count.value = 0;
            let body = serde_json::to_string(&*count).unwrap();
            let mut res = Response::new(Full::new(Bytes::from(body)));
            res.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("application/json"),
            );
            res
        }
        _ => {
            let mut res: Response<Full<Bytes>> = Response::new(Full::new(Bytes::from("Not Found")));
            *res.status_mut() = StatusCode::NOT_FOUND;
            res.headers_mut().insert(
                hyper::header::CONTENT_TYPE,
                hyper::header::HeaderValue::from_static("text/plain"),
            );
            res
        }
    };

    println!("\nResponse: ");
    println!("* Status: {}", res.status());
    println!("* Header:");
    for (k, v) in res.headers() {
        println!("    * {}: {:?}", k, v);
    }
    println!("* Body: {:?}", String::from_utf8_lossy(res.body().clone().collect().await.unwrap().to_bytes().as_ref()));
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const PORT: u16 = 3000;
    let addr = SocketAddr::from(([127, 0, 0, 1], PORT));
    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;
    
    println!("listening on http://{addr}");
    
    let count = Arc::new(Mutex::new(Count { value: 0 }));
    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let count = count.clone();
        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io,service_fn(move |req| handle_request(req, count.clone())))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
