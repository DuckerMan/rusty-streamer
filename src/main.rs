#[macro_use]
extern crate log;
#[macro_use]
extern crate simplelog;

use simplelog::*;
use std::fs::File;
use std::io;
use std::io::Stdin;
use tokio::runtime;

pub use stream::{make_server, start_client};

#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("logs.log").unwrap(),
        ),
    ])
    .unwrap();

    let basic_rt = runtime::Builder::new()
    .threaded_scheduler()
    .build().unwrap();


    info!("Start application");
    println!("Hi there! Do you want to start a client or server? Type it!");
    let stdin = io::stdin();
    let mut server = None;
    let mut client = None;

    'ask: loop {
        let mut buffer = String::new();
        stdin.read_line(&mut buffer);
        let buffer = buffer.trim();
        match buffer {
            "server" => {
                println!("Please, enter your server IP:port");
                let mut buffer = String::new();
                stdin.read_line(&mut buffer);
                let buffer = buffer.trim();
                server = Some(make_server(buffer));
                println!("Server is ready!");
                break 'ask;
            }
            "client" => {
                println!("Please, enter server IP:port");
                let mut buffer = String::new();
                stdin.read_line(&mut buffer);
                let buffer = buffer.trim();
                client = Some(start_client(buffer).await);
                println!("Started client");
                break 'ask;
            }
            "q" | "quit" => break 'ask,
            _ => {
                println!("Command `{}` isn't recognized", buffer);
                continue;
            }
        };
    }

    println!("Check..");
    if server.is_some() {
        server.unwrap().start();
    } else {
        let mut client = client.unwrap();

        let handle = tokio::spawn(async move {
            info!("Start new client thread");
            loop {
                let packet = client.get_packet().await;
                info!("Got packet with len {}", packet.len());
            }
        });

        handle.await;
    }
}
