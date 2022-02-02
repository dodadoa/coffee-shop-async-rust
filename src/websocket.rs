use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tungstenite::protocol::Message;

use crate::PeerMap;

pub async fn init_websocket(peer_map: PeerMap) {
    let addr = format!("localhost:3044");

    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let peer_map_new_thread = peer_map.clone();
        let (sender, receiver) = channel(5);
        let mut peer_map_mutex = peer_map_new_thread.lock().await;
        peer_map_mutex.insert(addr, sender);

        tokio::spawn(async move {
            
            println!("Incoming TCP connection from: {}", addr);

            let mut ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");
                
            println!("WebSocket connection established: {}", addr);
    
            let mut rx = ReceiverStream::new(receiver);
            while let Some(val) = rx.next().await {
                println!("Sending: {}", addr);
                ws_stream.send(Message::Text(val)).await.unwrap();
            }
        });
    }
}
