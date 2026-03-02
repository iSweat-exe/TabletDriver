use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::net::TcpListener;
use std::collections::HashMap;

use serde::{Serialize};
use tungstenite::{accept, Message};
use tungstenite::protocol::WebSocket;

use crate::input::SharedState;

#[derive(Serialize)]
struct WsPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pressure: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_connected: Option<bool>,
}

pub fn websocket_loop(shared: Arc<SharedState>) {
    let mut current_port = 0;
    let mut listener: Option<TcpListener> = None;
    let mut clients: HashMap<usize, WebSocket<std::net::TcpStream>> = HashMap::new();
    let mut next_client_id = 0;

    loop {
        // 1. Read current config
        let (enabled, port, hz, send_coords, send_pressure, _, send_status) = {
            let config = shared.config.read().unwrap();
            let ws = &config.websocket;
            (
                ws.enabled, 
                ws.port, 
                ws.polling_rate_hz.max(1), // Prevent div by 0
                ws.send_coordinates,
                ws.send_pressure,
                ws.send_tilt,
                ws.send_status
            )
        };

        // 2. Manage Server State
        if !enabled {
            if listener.is_some() {
                log::info!(target: "WebSocket", "WebSocket Server disabled, shutting down...");
                clients.clear();
                listener = None;
            }
        } else {
            // Check if port changed or listener needs starting
            if listener.is_none() || current_port != port {
                log::info!(target: "WebSocket", "Starting WebSocket Server on 127.0.0.1:{}", port);
                clients.clear();
                
                match TcpListener::bind(format!("127.0.0.1:{}", port)) {
                    Ok(l) => {
                        l.set_nonblocking(true).unwrap();
                        listener = Some(l);
                        current_port = port;
                    },
                    Err(e) => {
                        log::error!(target: "WebSocket", "Failed to bind to port {}: {}", port, e);
                        listener = None;
                    }
                }
            }
        }

        // 3. Process Server Logic if running
        if let Some(l) = &listener {
            // Accept new clients
            match l.accept() {
                Ok((stream, addr)) => {
                    log::info!(target: "WebSocket", "New connection from {}", addr);
                    stream.set_nonblocking(false).unwrap(); // Block for handshake
                    match accept(stream) {
                        Ok(mut websocket) => {
                            websocket.get_mut().set_nonblocking(true).unwrap(); // Non-blocking for data
                            clients.insert(next_client_id, websocket);
                            next_client_id += 1;
                        },
                        Err(e) => {
                            log::warn!(target: "WebSocket", "Error during WebSocket handshake: {}", e);
                        }
                    }
                },
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No new connections
                },
                Err(e) => {
                    log::error!(target: "WebSocket", "Listener error: {}", e);
                }
            }

            // Prepare Payload
            if !clients.is_empty() {
                let data = shared.tablet_data.read().unwrap().clone();
                
                let payload = WsPayload {
                    x: if send_coords { Some(data.x) } else { None },
                    y: if send_coords { Some(data.y) } else { None },
                    pressure: if send_pressure { Some(data.pressure) } else { None },
                    status: if send_status { Some(data.status.clone()) } else { None },
                    is_connected: if send_status { Some(data.is_connected) } else { None },
                };

                if let Ok(json) = serde_json::to_string(&payload) {
                    let mut dead_clients = vec![];
                    
                    for (id, client) in clients.iter_mut() {
                        if let Err(_) = client.send(Message::Text(json.clone().into())) {
                            dead_clients.push(*id);
                        }
                    }

                    for id in dead_clients {
                        clients.remove(&id);
                        log::info!(target: "WebSocket", "Client disconnected.");
                    }
                }
            }
        }

        // 4. Sleep according to polling rate
        let sleep_ms = 1000 / hz;
        thread::sleep(Duration::from_millis(sleep_ms as u64));
    }
}
