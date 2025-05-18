use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc::Receiver,
};
use std::sync::{Arc, Mutex};
use simple_websockets::{Event, EventHub, Responder, Message};
use std::collections::HashMap;
use itertools::Itertools;
use std::thread;
use crate::utils::*;
use crate::vprintln;


const ADDRESS: &str = "0.0.0.0";
const WEB_PORT: u16 = 8888;
const SOCKET_PORT: u16 = 8889;
const MAX_THREADS: usize = 4;

pub fn websocket_listen(receiver: Receiver<String>) {
    let event_hub = simple_websockets::launch(SOCKET_PORT)
        .expect("failed to listen on port {SOCKET_PORT}");
    let clients = Arc::new(Mutex::new(HashMap::<u64, Responder>::new()));

    let sender_clone = Arc::clone(&clients);
    let connection_clone = Arc::clone(&clients);

    thread::spawn(|| handle_websocket_connection(event_hub, connection_clone));
    handle_data_responder(receiver, sender_clone);
}

fn handle_data_responder(receiver: Receiver<String>, clients: Arc<Mutex<HashMap<u64, Responder>>>) {
    loop {
        match receiver.recv() {
            Ok(data) => {
                let responders = clients.lock().unwrap();
                for (_, responder) in responders.iter() {
                    responder.send(Message::Text(data.clone()));
                    vprintln!("sent to client: {data}");
                }
            }
            Err(_) => break,
        }
    }

}

fn handle_websocket_connection(event_hub: EventHub, clients: Arc<Mutex<HashMap<u64, Responder>>>) {
    loop {
        match event_hub.poll_event() {
            Event::Connect(client_id, responder) => {
                vprintln!("#{client_id} connected");

                let mut responders = clients.lock().unwrap();
                responders.insert(client_id, responder);
            },
            Event::Disconnect(client_id) => {
                vprintln!("#{client_id} disconnected");

                let mut responders = clients.lock().unwrap();
                responders.get(&client_id).unwrap().close();
                responders.remove(&client_id);
            },
            _ => {
                // no-op
            }
        }
    }
}

pub fn http_listen() {
    // starting listening
    let listener = TcpListener::bind(format!("{ADDRESS}:{WEB_PORT}")).unwrap();

    // open app in browser
    if open::that(format!("http://{ADDRESS}:{WEB_PORT}")).is_ok() {
        println!("Server running at {ADDRESS}:{WEB_PORT}");
    } else {
        println!("Failed to open application in browser.");
    }

    let thread_pool = threadpool::ThreadPool::new(MAX_THREADS);

    // listen for incoming
    loop {
        let (stream, _) = listener.accept().unwrap();

        thread_pool.execute(|| {
            handle_http_connection(stream);
        });
    }
}

fn handle_http_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let (method, end_point, _http_version) = http_request[0].split(" ").collect_tuple().unwrap();

    // assume all GET requests for now
    if method != "GET" {
        vprintln!("Unsupported method: {method}");
        write_response(&mut stream, 400, None, None);
        return;
    }

    vprintln!("Request: {method} {end_point}");

    match end_point {
        "/socket_port" => {
            write_response(&mut stream, 200, None, Some(SOCKET_PORT.to_string()));
        },
        _ => {
            // get file
            let end_point = if end_point == "/" {
                "/index.html"
            } else {
                end_point
            };

            let file = get_frontend(end_point);
            if file.is_ok() {
                write_response(&mut stream, 200, None, Some(file.unwrap()));
            } else {
                vprintln!("{method} {end_point} failed");
                vprintln!("{}", file.err().unwrap());
                write_response(&mut stream, 404, None, None);
            }
        }
    };

}

