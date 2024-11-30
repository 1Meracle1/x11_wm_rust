use std::{
    fs,
    io::Read,
    os::unix::net::UnixListener,
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

use log::{error, info};
use msg_types::WmCommand;

pub struct UnixServerWorker {
    thread_handle: Option<JoinHandle<()>>,
}

impl UnixServerWorker {
    pub fn new(socket_path: &str, sender: Sender<WmCommand>) -> Self {
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
                    match WmCommand::deserialize(message.into_bytes().as_slice()) {
                        Ok(decoded_message) => sender.send(decoded_message).unwrap(),
                        Err(err) => error!("Failed to accept message: {}", err),
                    };
                }
                Err(err) => error!("Connection failed: {}", err),
            };
        });
        Self {
            thread_handle: Some(thread_handle),
        }
    }
}

impl Drop for UnixServerWorker {
    fn drop(&mut self) {
        self.thread_handle.take().unwrap().join().unwrap();
    }
}
