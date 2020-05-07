// use crossbeam::channel::{unbounded, Receiver, Sender};
// use scrap::{Capturer, Display};
// use std::env;
// use std::io::prelude::*;
// use std::io::ErrorKind::WouldBlock;
// use std::io::{self, Write};
// use std::net::{TcpListener, TcpStream};
// use std::process;
// use std::thread;
// use std::time::{Duration, Instant};
// use tokio::net::TcpStream;
// use std::io::Error;
// use tokio::prelude::*;
// use std::convert::TryInto;

// type Stream = std::thread::JoinHandle<()>;
// type Frame = Vec<u8>;

// fn main() {
//     let threads = start_stream();
//     threads.0.join();
//     threads.1.join();
// }
// pub fn start_stream() -> (Stream, Stream) {
//     let (tx, rx) = unbounded();
//     (make_stream_thread(tx), make_transmitter_thread(rx))
// }

// pub fn make_transmitter_thread<'a>(rx: Receiver<Frame>) -> Stream {
//     let mut clients = Arc::new(RwLock::new(Vec::new()));
//     let clients_to_server = clients.clone();
//     let size = {
//         let display = Display::primary().unwrap();
//         let size = (display.width() * display.height() * 4) as u32;
//         println!("SIZEEEE: {}", size);
//         size.to_be_bytes()
//     };

//     println!(
//         "[{}], [{}], [{}], [{}], ",
//         size[0], size[1], size[2], size[3]
//     );
//     thread::spawn(move || {
//         let server = TcpListener::bind("127.0.0.1:8080").unwrap();
//         for client in server.incoming() {
//             println!("Got client for US!");
//             let mut client = client.unwrap();
//             client.set_nonblocking(true).unwrap();
//             client.set_nodelay(true).expect("set_nodelay call failed");
//             client.write(&size); // Send size to client
//             if clients_to_server.read().unwrap().len() == 0 {
//                 clients_to_server.write().unwrap().push(client);
//             } else {
//                 clients_to_server.write().unwrap()[0] = client;
//             }
//         }
//     });

//     thread::spawn(move || {
//         dbg!("made transmitter thread");
//         for data in rx.iter() {
//             if clients.read().unwrap().get(0).is_some() {
//                 // clients.write().unwrap()[0].write(&SPECIAL_DELMITER);
//                 clients.write().unwrap()[0].write(&data);
//                 println!("Send data to client with len: {}", data.len());
//             }
//         }
//     })
// }

use crossbeam::channel::{unbounded, Receiver, Sender};
use scrap::{Capturer, Display};
use std::io::Error;
use std::io::ErrorKind::WouldBlock;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use image::jpeg::JPEGEncoder;
use image::{ColorType, GenericImage, GenericImageView, ImageBuffer, RgbaImage};
use std::convert::TryInto;
use image::DynamicImage::ImageRgba8;
// use tokio::prelude::*;
type Stream = std::thread::JoinHandle<()>;
pub struct Server {
    socket: Arc<TcpListener>,
}

impl Server {
    pub fn new(ip: &str) -> Self {
        let mut socket = Arc::new(TcpListener::bind(ip).unwrap());
        Self { socket }
    }

    pub fn start(&mut self) {
        info!("Start server!");
        let (tx, rx) = unbounded();
        self.make_sender_thread(rx);
        let th = self.make_stream_thread(tx);
        th.join();
    }

    fn make_stream_thread(&self, tx: Sender<Vec<u8>>) -> Stream {
        thread::spawn(move || {
            info!("Make stream thread");

            let display = Display::primary().expect("Couldn't find primary display.");
            let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
            let (w, h) = (capturer.width(), capturer.height());
            loop {
                let start = Instant::now();
                let buffer = match capturer.frame() {
                    Ok(buffer) => buffer,
                    Err(error) => {
                        if error.kind() == WouldBlock {
                            // Keep spinning.
                            continue;
                        } else {
                            panic!("Error: {}", error);
                        }
                    }
                };

                let mut bitflipped = Vec::with_capacity(w * h * 4);
                let stride = buffer.len() / h;

                for y in 0..h {
                    for x in 0..w {
                        let i = stride * y + 4 * x;
                        bitflipped.extend_from_slice(&[buffer[i], buffer[i + 1], buffer[i + 2]]);
                    }
                }
                
                
                
                let img = image::ImageBuffer::from_raw(w as u32,h as u32,Vec::from(&*buffer)).unwrap();
                let img = image::DynamicImage::ImageBgra8(img);
                
                let mut data: Vec<u8> = Vec::new();
                let enc = Instant::now();
                let mut encoder = JPEGEncoder::new_with_quality(&mut data, 1);
                info!("Made encoder for {:#?}", enc.elapsed());
                encoder
                .encode(
                        &bitflipped,
                        w.try_into().unwrap(),
                        h.try_into().unwrap(),
                        ColorType::Rgb8,
                    )
                    .unwrap();
                    
                    data.flush();
                    
                info!("Captured! Took {:#?}", start.elapsed());
                tx.send(data);
                // dbg!(buffer);
                // break;
            }
        })
    }

    fn make_sender_thread(&mut self, rx: Receiver<Vec<u8>>) {
        let listener = self.socket.clone();

        let mut client = Arc::new(RwLock::new(Vec::new()));

        let client_for_get = client.clone();
        let client_for_send = client.clone();

        thread::spawn(move || {
            info!("Start new sender thread");
            for connection in listener.incoming() {
                info!("Got a new connection!");
                let connection = connection.unwrap();
                if client_for_get.read().unwrap().get(0).is_none() {
                    client_for_get.write().unwrap().push(connection);
                } else {
                    client_for_get.write().unwrap()[0] = connection;
                }
            }
        });

        thread::spawn(move || {
            for data in rx.iter() {
                if client_for_send.read().unwrap().get(0).is_some() {
                    let mut c = &client_for_send.write().unwrap()[0];
                    let size = (data.len() as u32).to_be_bytes();
                    info!("Data is : {:#?}", &size);
                    c.write(&size);
                    c.write(&data);
                    info!("Send data to client with len {}", data.len());
                }
            }
        });
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        println!("Server is dropped");
    }
}

pub fn make_server(ip: &str) -> Server {
    info!("Preparing to make server");
    let mut server = Server::new(ip);
    server
}
