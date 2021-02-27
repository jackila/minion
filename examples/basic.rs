extern crate minion;

use minion::Cancellabel;
use std::thread;
use std::{
    io::{self, prelude::*},
    net, time,
};

struct Service(net::TcpListener);
impl minion::Cancellabel for Service {
    type Error = io::Error;
    fn for_each(&mut self) -> Result<minion::LoopState, Self::Error> {
        let mut stream = self.0.accept()?.0;
        write!(stream, "hello")?;
        stream.shutdown(net::Shutdown::Both)?;
        Ok(minion::LoopState::Continue)
    }
}

impl Service {
    fn new() -> Self {
        let listener = net::TcpListener::bind("127.0.0.1:6556").unwrap();
        Service(listener)
    }
}
fn main() {
    let s = Service::new();
    eprintln!("server is running");
    let h = s.spawn();
    let exit = h.canceller();
    thread::spawn(move || {
        thread::sleep(time::Duration::from_secs(10));
        eprintln!("server termining");
        exit.cancle();
    });
    h.wait().unwrap();
    eprintln!("server terminated");
}
