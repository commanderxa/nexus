use std::{error::Error, net::SocketAddr, sync::Arc};

use tokio::{
    net::UdpSocket,
    sync::{mpsc, Mutex},
};

use nexuslib::models::call::media_call::MediaCall;

use crate::state::connection::ConnectionState;

/// Handles UDP stream for calls
pub async fn handle_udp(
    sock: UdpSocket,
    state: Arc<Mutex<ConnectionState>>,
) -> Result<(), Box<dyn Error>> {
    let r = Arc::new(sock);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<(MediaCall, SocketAddr)>(1_000);

    let mut buf = [0; 1024];
    tokio::select! {
        // received a call from a peer
        Some((call, addr)) = rx.recv() => {
            // transmit the call frame to the session receiver
            let len = s.send_to(&call.message, addr).await.unwrap();
            println!("{:?} bytes sent", len);
        }
        // received a call frame from a user
        result = r.recv_from(&mut buf) => match result {
            Ok((len, addr)) => {
                println!("{:?} bytes received from {:?}", len, addr);

                // deserializing call and extracting the receiver
                let call = MediaCall::from_bytes(buf.to_vec());
                let receiver = &call.sides.get_receiver().to_owned();
                let receiver_peer = &call.peers.get_receiver().to_owned().unwrap();
                let recv_addr = state.lock().await.peers
                                    .get(receiver).unwrap()
                                    .get(receiver_peer).unwrap()
                                    .socket_addr;

                // sending the call frame to the receiver session peer
                tx.send((call.clone(), recv_addr)).await.unwrap();
            }
            Err(e) => {
                log::error!("Got error while receiving UDP Stream! {e}");
            },
        }
    }

    Ok(())
}
