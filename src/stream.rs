// stream.rs
use tokio::{net::TcpStream, io::{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use tokio::sync::mpsc;
use std::collections::HashMap;


pub struct ConnectionHandle {
    pub tx: mpsc::UnboundedSender<ConnCommand>,
}

pub enum ConnCommand {
    SendLine(String),
    Quit,
}

pub type ServerId = String;

#[derive(Debug)]
pub enum NetEvent {
    Line(String),
    Error(String),
}

#[derive(Default)]
pub struct StreamManager {
    conns: HashMap<ServerId, ConnectionHandle>,
}

impl StreamManager {
    pub async fn connect(&mut self, server_id: ServerId, addr: String, net_tx: mpsc::UnboundedSender<(ServerId, NetEvent)>, nick: String, real: String, oauth: String) -> bool{
        let (tx, mut rx) = mpsc::unbounded_channel();
        let net_tx2 = net_tx.clone();
        let sid = server_id.clone();
        tokio::spawn(async move {
            let stream = match TcpStream::connect(&addr).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = net_tx2.send((sid.clone(), NetEvent::Error(format!("Failed to connect: {e}"))));
                    return;
                }
            };
            let (r, mut w) = stream.into_split();
            let mut reader = BufReader::new(r).lines();

            let w_oauth = "PASS oauth:".to_owned() + &oauth + "\r\n";
            let cap_user = "CAP REQ :twitch.tv/membership".to_owned() + "\r\n";
            let w_nick = "NICK ".to_owned() + &nick + "\r\n";
            let w_real = "USER guest 0 * :".to_owned() + &real + "\r\n";

            if oauth.is_empty() {
                let _ = w.write_all(w_nick.as_bytes()).await;
                let _ = w.write_all(w_real.as_bytes()).await;
            } else {
                let _ = w.write_all(w_oauth.as_bytes()).await;
                let _ = w.write_all(w_nick.as_bytes()).await;
                let _ = w.write_all(cap_user.as_bytes()).await;
            }

            loop {
                tokio::select! {
                    Some(cmd) = rx.recv() => match cmd {
                        ConnCommand::SendLine(s) => {
                            let _ = w.write_all(s.as_bytes()).await;
                            let _ = w.write_all(b"\r\n").await; 
                        }
                        ConnCommand::Quit => break,
                    },
                    result = reader.next_line() => {
                        match result {
                            Ok(Some(line)) => {
                                let _ = net_tx2.send((sid.clone(), NetEvent::Line(line)));
                            }
                            Ok(None) => {
                                let _ = net_tx2.send((sid.clone(), NetEvent::Error("Disconnected".to_string())));
                                break;
                            }
                            Err(e) => {
                                let _ = net_tx2.send((sid.clone(), NetEvent::Error(format!("Read error: {e}"))));
                                break;
                            }
                        }
                    }
                }
            }
        });

        self.conns.insert(server_id, ConnectionHandle { tx });
        true
    }

    pub fn send_line(&self, server_id: String, line:String) {
        if let Some(conn) = self.conns.get(&server_id) {
            let _ = conn.tx.send(ConnCommand::SendLine(line));
        }
    }
    pub fn disconnect(&mut self, server_id: &str) {
        if let Some(conn) = self.conns.remove(server_id) {
            // Send the Quit command to the task
            let _ = conn.tx.send(ConnCommand::Quit);
        }
    }
    pub fn disconnect_all(&mut self) {
        let server_ids: Vec<ServerId> = self.conns.keys().cloned().collect();
        for sid in server_ids {
            self.disconnect(&sid);
        }
    }

}

