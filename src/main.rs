use axum::{
    handler::{get, post},
    response::IntoResponse,
    Json, Router
};
use axum::response::Html;
use cyberdeck::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use lazy_static::*;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref CONNECTIONS: Mutex<HashMap<String, Cyberdeck>> = Mutex::new(HashMap::new());
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/api/connect", post(connect));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    println!("http://localhost:3000");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> impl IntoResponse {
    Html(include_str!("./index.html"))
}

const MAX_WIDTH:i32 = 100;
const MAX_HEIGHT:i32 = 28;

async fn connect(Json(payload): Json<ConnectRequest>) -> impl IntoResponse {
    let id = Uuid::new_v4();

    let mut cd = Cyberdeck::new(|e| async move {
        match e {
            CyberdeckEvent::DataChannelMessage(c, m) => {
                println!("Recieved a message from channel {}!", c.name());
                let msg_str = String::from_utf8(m.data.to_vec()).unwrap();
                println!("Message from DataChannel '{}': {}", c.name(), msg_str);
            }
            CyberdeckEvent::DataChannelStateChange(c) => {
                if c.state() == RTCDataChannelState::Open {
                    println!("DataChannel '{}' opened", c.name());
                    c.send_text(r###"..............
..............
..............
.......#......
..............
.............."###).await.unwrap();
                } else if c.state() == RTCDataChannelState::Closed {
                    println!("DataChannel '{}' closed", c.name());
                }
            }
            CyberdeckEvent::PeerConnectionStateChange(s) => {
                println!("Peer connection state: {} ", s)
            }
        }
    })
    .await
    .unwrap();
    let answer = cd.receive_offer(&payload.offer).await;

    let mut conns = CONNECTIONS.lock().unwrap();

    conns.insert(id.to_string(), cd);

    Json(ConnectResponse {
        id: id.to_string(),
        answer: answer.unwrap().clone(),
    })
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct ConnectRequest {
    offer: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct ConnectResponse {
    id: String,
    answer: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}
