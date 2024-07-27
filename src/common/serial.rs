use std::io::Error;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_serial::SerialStream;

pub struct Serial {
    send: Sender<Vec<u8>>,
    receive: Receiver<Vec<u8>>,
    run_flag: Arc<Mutex<bool>>,
}

impl Serial {

    pub fn receive<F>(&mut self, future: F)
    where
        F: FnOnce(Vec<u8>),
    {
        
    }

    pub fn close(&mut self) {
        let flag_arc = Arc::clone(&self.run_flag);
        let mut flag = (*flag_arc).lock().unwrap();
        *flag = false;
    }
}

pub fn new(port: SerialStream) -> Result<Serial, Error> {
    let (tx, rx) = mpsc::channel::<Vec<u8>>(1024);
    Ok(Serial {
        send: tx,
        receive: rx,
        run_flag: Arc::new(Mutex::new(true)),
    })
}
