mod frontend;
mod ssm2;
mod utils;

use frontend::*;
use json::object;
use serialport::{SerialPortType::UsbPort, available_ports};
use ssm2::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, sleep};
use std::time::Duration;

fn main() {
    //     let mut ssm2 = Ssm2::new(&("".to_string()));

    //     let buf = &mut Vec::new();

    //     ssm2.ecu_read(EcuParam::EngineSpeed, buf);
    //     return;

    // Broadcast channel between serial port and frontend
    let (sender, receiver) = mpsc::channel::<String>();
    init_frontend(receiver);
    init_serial_port(sender);
}

const MANUFACTURER: &str = "FTDI";
fn init_serial_port(sender: Sender<String>) {
    // // data mock
    // let mut count = 0;

    // let mut data_message = object!{
    //     ecu1: 0,
    //     ecu2: 0,
    // };

    // loop {
    //     sender.send(data_message.to_string()).unwrap();
    //     count += 1;
    //     if count > 100 {
    //         count = 0;
    //     }
    //     data_message["ecu1"] = count.into();
    //     data_message["ecu2"] = (50 - count).into();
    //     data_message["ecu3"] = (50 - count).into();
    //     sleep(Duration::from_millis(100));
    // }

    let mut port_name: String = String::new();

    match available_ports() {
        Ok(ports) => {
            let mut found = false;

            for p in ports {
                vprintln!("Found port {}, {:?}", p.port_name, p.port_type);
                match p.port_type {
                    UsbPort(port_info) => {
                        if port_info.manufacturer.unwrap_or_default() == MANUFACTURER {
                            println!("Found port at {}", p.port_name);
                            port_name = p.port_name;
                            found = true;
                            break;
                        }
                    }
                    _ => {
                        // no-op
                    }
                }
            }
            if !found {
                println!("FTDI serial port not found!");
                return;
            }
        }
        Err(e) => {
            eprintln!("Error listing ports: {}", e);
            return;
        }
    }

    // ssm2
    let mut ssm2 = Ssm2::new(&port_name);
    ssm2.open();

    let buf = &mut Vec::new();
    ssm2.ecu_init(buf);

    println!("Init response: {:02X?}", buf);

    println!("Reading ECU data...");

    let mut data_message = object! {
        ecu1: 0,
        ecu2: 0,
        ecu3: 0,
    };

    loop {
        // temp
        ssm2.ecu_read(EcuParam::IntakeTemp, buf);
        data_message["ecu1"] = (buf[5] - 40).into();
        ssm2.ecu_read(EcuParam::ThrottleAngle, buf);
        data_message["ecu2"] = buf[5].into();
        ssm2.ecu_read(EcuParam::AirFuelSensor1, buf);
        data_message["ecu3"] = buf[5].into();
        sender.send(data_message.to_string()).unwrap();

        sleep(Duration::from_millis(100));
    }
}

fn init_frontend(receiver: Receiver<String>) {
    thread::spawn(|| http_listen());
    thread::spawn(|| websocket_listen(receiver));
}
