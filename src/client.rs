
use tokio::net::TcpStream;
use std::io::Error;
use tokio::prelude::*;
use std::convert::TryInto;

pub struct Client {
    data: Vec<u8>,
    socket: TcpStream
}

impl Client {
    pub async fn new(ip: &str) -> Result<Self, Error>{
        info!("Trying to start connection with ip {}", ip);
        let socket = TcpStream::connect(ip).await;
        if socket.is_err() {
            let err = socket.err().unwrap();
            error!("Error while trying to connect: {:?}", err);
            return Err(err);
        }
        let socket = socket.unwrap();
        info!("Connected!");
        Ok(Self{socket, data: Vec::new()})
    }

    pub async fn get_packet(&mut self) -> Vec<u8> {
        // let(r, w) = self.socket.split();
        let size = {
            let mut buf = [0u8; 4];
            self.socket.read(&mut buf).await;
            u32::from_be_bytes(buf).try_into().unwrap()
        };

        let mut data: Vec<u8> = vec![0; size];
        self.socket.read_exact(&mut data).await;
        info!("Read data from socket with {} length", data.len());
        data
    }
}

pub async fn start_client(ip: &str) -> Client {
    println!("------------------- Trying to connect -------------------");
    info!("Trying to connect to {}", ip);
    let client = Client::new(ip).await;
    if client.is_err(){
        let err = client.err();
        error!("Error when trying to connect! {:?}", err);
        panic!();
    }

    let mut client = client.unwrap();
    client
}