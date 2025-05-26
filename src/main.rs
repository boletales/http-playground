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
    match (req.method(), req.uri().path()) {
        // クライアントページ
        (&Method::GET, "/") => {
            let html = include_str!("../static/index.html");
            Ok(Response::new(Full::new(Bytes::from(html))))
        }
        // カウント取得
        (&Method::GET, "/getcount") => {
            let count = count.lock().unwrap().clone();
            let body = serde_json::to_string(&count).unwrap();
            Ok(Response::new(Full::new(Bytes::from(body))))
        }
        // カウント加算
        (&Method::POST, "/addcount") => {
            let whole_body = req.into_body().collect().await.unwrap().to_bytes();
            let add: Count = serde_json::from_slice(&whole_body).unwrap_or(Count { value: 0 });
            let mut count = count.lock().unwrap();
            count.value += add.value;
            let body = serde_json::to_string(&*count).unwrap();
            Ok(Response::new(Full::new(Bytes::from(body))))
        }
        // カウントリセット
        (&Method::POST, "/resetcount") => {
            let mut count = count.lock().unwrap();
            count.value = 0;
            let body = serde_json::to_string(&*count).unwrap();
            Ok(Response::new(Full::new(Bytes::from(body))))
        }
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
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
