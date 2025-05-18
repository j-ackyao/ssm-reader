mod frontend;
mod utils;
mod ssm2;

use std::thread::{self, sleep};
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::Duration;
use json::object;
use ssm2::Ssm2;
use frontend::*;

fn main() {
    // Broadcast channel between serial port and frontend
    let (sender, receiver) = mpsc::channel::<String>();
    
    init_frontend(receiver);
    init_serial_port(sender);
}

fn init_serial_port(sender: Sender<String>) {
    // data mock
    let mut count = 0;
    
    let mut data_message = object!{
        ecu1: 0,
        ecu2: 0,
    };

    loop {
        sender.send(data_message.to_string()).unwrap();        
        count += 1;
        if count > 100 {
            count = 0;
        }
        data_message["ecu1"] = count.into();
        data_message["ecu2"] = (50 - count).into();
        data_message["ecu3"] = (50 - count).into();
        sleep(Duration::from_millis(100));
    }

    // ssm2
    let ssm2 = Ssm2::new(&"/dev/ttyUSB0".to_string());

    ssm2.open();

}

fn init_frontend(receiver: Receiver<String>) {
    thread::spawn(|| { http_listen() });
    thread::spawn(|| { websocket_listen(receiver) });
}
