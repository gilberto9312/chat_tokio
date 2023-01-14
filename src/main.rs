use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener, sync::broadcast
};
use std::env;
use log::{info, warn};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let listener = TcpListener::bind(&args[1]).await.unwrap();
    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(10);

    loop{
        let (mut socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                  _ = reader.read_line(&mut line) => {
                    let send = tx.send((line.clone(), addr));
                    match send {
                        Ok(_) => {
                            line.clear();
                        }
                        Err(e)=> {
                            info!("Error reader: {}", e);
                        }
                    }
                  }
                  result = rx.recv() => {
                    match result {
                        Ok(_) => {
                            let (msg, msg_addr) = result.expect("can't get");
                            if addr != msg_addr {
                                writer.write_all(msg.as_bytes()).await.expect("can't write");
                              }
                        }
                        Err(e) => {
                            warn!("error result: {}", e);
                        }
                    }
                    
                    
                  }
                }
            }
        });       
    }
}
