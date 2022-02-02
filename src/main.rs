use actix_web::{web, App, HttpServer, HttpResponse};
use anyhow::Result;
use barista::{init_baristas, Barista, retrieve_available_barista_id};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use tokio::time::sleep;
use std::time::Duration;

pub mod websocket;
pub mod barista;

pub type PeerMap = Arc<Mutex<HashMap<SocketAddr, Sender<String>>>>;

pub async fn purchase_endpoint(
    data_baristas: web::Data<Arc<Mutex<Barista>>>,
    data_peer_map: web::Data<PeerMap>,
) -> HttpResponse {

    let available_barista_id =
        retrieve_available_barista_id(data_baristas.clone()).await;

    if available_barista_id.is_none() {
        return HttpResponse::InternalServerError().json(json!({
            "error": "no barista availables now"
        }));
    }

    tokio::spawn(async move {
        sleep(Duration::from_millis(3000)).await;

        println!("Peer Map: {:?}", data_peer_map);

        let data_sender_mutex = data_peer_map.lock().await;
        let data_senders = data_sender_mutex.values();

        println!("Sender: {:?}", data_senders);
        for data_sender in data_senders {
            data_sender
            .send(
                json!({
                    "account": available_barista_id.unwrap(),
                })
                .to_string(),
            )
            .await
            .unwrap();
        }
    });

    HttpResponse::Ok().json(json!({
        "receiverAddress": available_barista_id.unwrap()
    }))
}


pub async fn run_server() -> Result<()> {
    let peer_map: PeerMap = Arc::new(Mutex::new(HashMap::new()));
    let barista = Arc::new(Mutex::new(init_baristas().await.unwrap()));
    
    let peer_map_websocket = peer_map.clone();
    tokio::spawn(async move {
        websocket::init_websocket(peer_map_websocket).await;
    });

    let peer_map_data = web::Data::new(peer_map.clone());
    let barista_data = web::Data::new(barista);
    
    HttpServer::new(move || {
        App::new()
            .app_data(peer_map_data.clone())
            .app_data(barista_data.clone())
            .route(
                "/latte/purchase",
                web::post().to(purchase_endpoint),
            )
    })
    .bind("localhost:8080")?
    .run()
    .await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> Result<()> {
    run_server().await?;
    Ok(())
}
