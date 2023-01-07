use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let peers = PeerMap::new(Mutex::new(HashMap::new()));

    info!("Starting Kimchi server");

    let server = TcpListener::bind("0.0.0.0:4000").await?;

    let server_addr = server.local_addr().expect("Should have a local address");
    info!("Server listening on {}", server_addr);

    while let Ok((socket, addr)) = server.accept().await {
        tokio::spawn(process_socket(socket, addr, peers.clone()));
    }

    Ok(())
}

async fn process_socket(socket: TcpStream, addr: SocketAddr, peers: PeerMap) {
    info!("Incoming TCP connection from: {}", addr);

    let ws = tokio_tungstenite::accept_async(socket)
        .await
        .expect("Error during the websocket handshake");
    info!("Websocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peers.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        info!(
            "Received a message from {}: {}",
            addr,
            msg.to_text().unwrap()
        );
        let peerz = peers.lock().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients = peerz
            .iter()
            .filter(|(peer_addr, _)| peer_addr != &&addr)
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    info!("{} disconnected", &addr);
    peers.lock().unwrap().remove(&addr);
}
