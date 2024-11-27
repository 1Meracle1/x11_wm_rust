use std::{
    fs,
    io::Read,
    os::unix::net::UnixListener,
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

use log::{error, info};

#[derive(Debug)]
pub enum UnixClientMessage {
    FocusLeft,
    FocusRight,
}

pub struct UnixServerWorker {
    thread_handle: Option<JoinHandle<()>>,
}

impl UnixServerWorker {
    pub fn new(socket_path: &str, sender: Sender<UnixClientMessage>) -> Self {
        if fs::metadata(socket_path).is_ok() {
            fs::remove_file(socket_path).unwrap();
        }
        let listener = UnixListener::bind(socket_path).unwrap();
        info!("Unix Server listening to {}", socket_path);
        let thread_handle = thread::spawn(move || loop {
            match listener.accept() {
                Ok((mut stream, _addr)) => {
                    let mut message = String::new();
                    stream.read_to_string(&mut message).unwrap();
                    if let Some(decoded_message) = decode_unix_client_message(&message) {
                        sender.send(decoded_message).unwrap();
                    } else {
                        error!("Failed to decode message: {}", message);
                    }
                }
                Err(err) => error!("Connection failed: {}", err),
            };
        });
        Self {
            thread_handle: Some(thread_handle),
        }
    }
}

fn decode_unix_client_message(message: &String) -> Option<UnixClientMessage> {
    let parts: Vec<&str> = message.split(' ').collect();
    if !parts.is_empty() {
        if parts.first().unwrap().starts_with("focus") && parts.len() == 2 {
            let focus_type = parts.get(1).unwrap();
            if focus_type.starts_with("left") {
                return Some(UnixClientMessage::FocusLeft);
            } else if focus_type.starts_with("right") {
                return Some(UnixClientMessage::FocusRight);
            }
        }
    }
    None
}

impl Drop for UnixServerWorker {
    fn drop(&mut self) {
        self.thread_handle.take().unwrap().join().unwrap();
    }
}
