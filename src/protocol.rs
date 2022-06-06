use bytes::{Buf, BufMut, BytesMut};

use serde_json::error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
// use tokio::net::TcpStream;

use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::io::Read;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::{self, UnboundedSender};
#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[repr(C, packed)]
#[derive(Default)]
pub struct Header {
    pub service_code: u32,
    pub length: u32,
}
#[repr(C, packed)]
#[derive(Deserialize, PartialEq, Debug, Default)]
pub struct Protocol {
    pub header: Header,
    pub body: Vec<u8>,
}
const HEADER_SIZE: usize = 8;
pub struct Parser {
    pub byteQueue: bytes::BytesMut,
}
impl Parser {
    pub fn new() -> Parser {
        Parser {
            byteQueue: BytesMut::with_capacity(64),
        }
    }
    pub fn parse(&mut self) -> Result<Protocol, Box<dyn Error>> {
        let mut result: Protocol = Default::default();
        if self.byteQueue.len() >= HEADER_SIZE {
            // let ttt = self.byteQueue.reader();
            let r = &self.byteQueue[0..HEADER_SIZE];
            let header = bincode::deserialize::<Header>(r)?;
            if self.byteQueue.len() >= HEADER_SIZE + header.length as usize {
                let mut v = [0u8; HEADER_SIZE];
                self.byteQueue.copy_to_slice(&mut v[..]); // 버리기
                let mut v = vec![0u8; header.length as usize];
                self.byteQueue.copy_to_slice(&mut v[..]);
                result.header = header;
                result.body = v;
                return Ok(result);
            }
        }
        Err("yet".into())
    }
}

#[test]
fn it_works() {
    let mut p = Parser::new();
    let clientData = vec![
        0x01u8, 0x00, 0x00, 0x00, 0x05, 0x0, 0x0, 0x0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x01u8, 0x00,
        0x00, 0x00, 0x05, 0x0, 0x0, 0x0, 0x11, 0x22, 0x33, 0x44, 0x55,
    ];
    p.byteQueue.put(&clientData[..]);

    let header = p.parse();
    let header = match header {
        Ok(v) => v,
        Err(e) => {
            return ();
        }
    };
    assert_eq!(header.header.service_code, 1);
    assert_eq!(header.header.length, 5);
    assert_eq!(header.body[0..5], vec![0x11u8, 0x22, 0x33, 0x44, 0x55]);

    let header = p.parse();
    let header = match header {
        Ok(v) => v,
        Err(e) => {
            return ();
        }
    };
    assert_eq!(header.header.service_code, 1);
    assert_eq!(header.header.length, 5);
    assert_eq!(header.body[0..5], vec![0x11u8, 0x22, 0x33, 0x44, 0x55]);
    // assert_eq!(2 + 2, 46);
    // assert_eq!(2 + 2, 46);
}
#[test]
fn lack_of_header() {
    let mut p = Parser::new();

    let clientData = vec![0x01u8, 0x00, 0x00, 0x00, 0x05, 0x0];
    p.byteQueue.put(&clientData[..]);

    let header = p.parse();
    assert_eq!(header.is_err(), true);

    p.byteQueue.put(
        &vec![
            0x0, 0x0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x01u8, 0x00, 0x00, 0x00, 0x05, 0x0, 0x0, 0x0,
            0x11, 0x22, 0x33, 0x44, 0x55,
        ][..],
    );
    let header = p.parse();
    assert_eq!(header.is_err(), false);
    let header = match header {
        Ok(v) => v,
        Err(e) => return (),
    };
    assert_eq!(header.header.service_code, 1);
    assert_eq!(header.header.length, 5);
    assert_eq!(header.body[0..5], vec![0x11u8, 0x22, 0x33, 0x44, 0x55]);

    let header = p.parse();
    let header = match header {
        Ok(v) => v,
        Err(e) => {
            return ();
        }
    };
    assert_eq!(header.header.service_code, 1);
    assert_eq!(header.header.length, 5);
    assert_eq!(header.body[0..5], vec![0x11u8, 0x22, 0x33, 0x44, 0x55]);

    let header = p.parse();
    let header = match header {
        Ok(v) => {
            assert_eq!(true, false);
        }
        Err(e) => {
            println!("empty: {:?}", e);
            assert_eq!(true, true);
            return ();
        }
    };
}
