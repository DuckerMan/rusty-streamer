#[macro_use] extern crate log;
#[macro_use] extern crate simplelog;

mod client;
mod server;

pub use client::Client;
pub use client::start_client;
pub use server::make_server;

pub trait Start {
    fn new(ip: &str) -> Self where Self: std::marker::Sized;
}