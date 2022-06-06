#![warn(rust_2018_idioms)]

use bytes::{Buf, BufMut, BytesMut};

use rustprac::protocol;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
// use tokio::net::TcpStream;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::{self, UnboundedSender};

fn service_1(protocol: &protocol::Protocol, sender: UnboundedSender<Vec<u8>>) {
    sender.send("i'am service_1".as_bytes().to_vec()).unwrap();
}
fn service_2(protocol: &protocol::Protocol, sender: UnboundedSender<Vec<u8>>) {
    sender.send("i'am service_2".as_bytes().to_vec()).unwrap();
}
fn service_3(protocol: &protocol::Protocol, sender: UnboundedSender<Vec<u8>>) {
    sender.send("i'am service_3".as_bytes().to_vec()).unwrap();
}
async fn session_processor(mut socket: OwnedReadHalf, sender: UnboundedSender<Vec<u8>>) {
    let mut parser = protocol::Parser::new();
    loop {
        let mut buf = vec![0; 2];
        let n = socket
            .read(&mut buf)
            .await
            .expect("failed to read data from socket");

        if n == 0 {
            return;
        }
        parser.byteQueue.put(&buf[..n]);
        loop {
            let p = parser.parse();
            let p = match p {
                Ok(v) => {
                    println!("parse ok");
                    v
                }
                Err(e) => {
                    println!("parse error: {:?}", e);
                    break;
                }
            };
            println!("tcp recv: {:?}", p);
    let functor: HashMap<u32, Box<dyn Fn(&protocol::Protocol, UnboundedSender<Vec<u8>>)>> =
        HashMap::new();
            let f = functor.get(&p.header.service_code);
            let f = match f {
                Some(v) => v,
                None => {
                    break;
                }
            };
            f(&p, sender.clone());
            if p.header.service_code == 1 {
                service_1(&p, sender.clone());
            } else if p.header.service_code == 2 {
                service_2(&p, sender.clone());
            } else if p.header.service_code == 3 {
                service_3(&p, sender.clone());
            }
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Allow passing an address to listen on as the first argument of this
    // program, but otherwise we'll just set up our TCP listener on
    // 127.0.0.1:8080 for connections.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (socket, _) = listener.accept().await?;
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        let (srx, mut stx) = socket.into_split();
        tokio::spawn(async move {
            loop {
                let t = rx.recv().await;
                match t {
                    Some(n) => {
                        println!("channel recv: data: {:?}", n);
                        stx.write_all(&n[..]).await.unwrap();
                    }
                    None => {
                        println!("channel is closed");
                        break;
                    }
                }
            }
        });
        // tokio::spawn(session_processor(srx, tx.clone(), funtor));
        tokio::spawn(session_processor(srx, tx.clone()));
        println!("two thread end?");
    }
}

            // let ff = funtor.lock().unwrap();
            // let ff = *ff;
            // let funtor: Arc<
            //     Mutex<HashMap<u32, &dyn Fn(&protocol::Protocol, UnboundedSender<Vec<u8>>)>>,
            // > = Arc::new(Mutex::new(HashMap::new()));
            // funtor.lock().unwrap().insert(1, &service_1);
            // funtor.lock().unwrap().insert(1, &service_2);
            // funtor.lock().unwrap().insert(1, &service_3);