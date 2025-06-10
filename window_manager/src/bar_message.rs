use std::{
    io::{Read, Write},
    os::{fd::{AsRawFd, RawFd}, unix::net::UnixStream},
};

use log::{trace, warn};

const MESSAGE_KEYBOARD_LAYOUT_TAG: u8 = 0;
const MESSAGE_WORKSPACE_LIST_TAG: u8 = 1;
const MESSAGE_WORKSPACE_ACTIVE_TAG: u8 = 2;
const MESSAGE_REQUEST_CLIENT_INIT_TAG: u8 = 3;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message<'a> {
    KeyboardLayout(&'a str),
    WorkspaceList(Vec<u32>),
    WorkspaceActive(u32),
    RequestClientInit,
}

impl<'a> Message<'a> {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // first 8 bytes for size of the message
        bytes.extend_from_slice(&[0u8; 8]);

        // next 1 byte is the integer value of the Message type tag,
        // followed by actual value
        match self {
            Message::KeyboardLayout(name) => {
                bytes.push(MESSAGE_KEYBOARD_LAYOUT_TAG);
                bytes.extend_from_slice(name.as_bytes());
            }
            Message::WorkspaceList(workspaces) => {
                bytes.push(MESSAGE_WORKSPACE_LIST_TAG);
                bytes.extend_from_slice(&workspaces.len().to_le_bytes());
                for workspace in workspaces {
                    bytes.extend_from_slice(&workspace.to_le_bytes());
                }
            }
            Message::WorkspaceActive(id) => {
                bytes.push(MESSAGE_WORKSPACE_ACTIVE_TAG);
                bytes.extend_from_slice(&id.to_le_bytes());
            }
            Message::RequestClientInit => {
                bytes.push(MESSAGE_REQUEST_CLIENT_INIT_TAG);
            }
        };

        // write actual size value in the first 8 bytes
        let message_size = (bytes.len() - 8) as u64;
        bytes[0..8].copy_from_slice(&message_size.to_le_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }
        // first 1 byte is the integer value of the Message type tag
        let msg_type_tag = bytes[0];
        match msg_type_tag {
            MESSAGE_REQUEST_CLIENT_INIT_TAG => Some(Message::RequestClientInit),
            _ => None,
        }
    }

    pub fn read_from_unix_stream(unix_stream: &mut UnixStream) -> Option<Self> {
        let mut buffer = [0; 4096];
        match unix_stream.read(&mut buffer[..size_of::<usize>()]) {
            Ok(n_size_bytes) => {
                if n_size_bytes == size_of::<usize>() {
                    if let Ok(arr) = buffer[..size_of::<usize>()].try_into() {
                        let message_size = usize::from_le_bytes(arr);
                        match unix_stream.read(&mut buffer[..message_size]) {
                            Ok(n_bytes) => Message::from_bytes(&buffer[..n_bytes]),
                            Err(err) => {
                                warn!(
                                    "failed to read message payload of size: {}, err: {}",
                                    message_size, err
                                );
                                None
                            }
                        }
                    } else {
                        warn!(
                            "failed to get message size from bytes: {:?}",
                            &buffer[..size_of::<usize>()]
                        );
                        None
                    }
                } else {
                    warn!(
                        "received message size in bytes that != size_of::<usize>(): {}",
                        n_size_bytes
                    );
                    None
                }
            }
            Err(err) => {
                warn!("failed to get message size from unix stream: {}", err);
                None
            }
        }
    }
}

pub struct UnixClients {
    unix_clients: Vec<UnixStream>,
}

impl UnixClients {
    pub fn new() -> Self {
        Self {
            unix_clients: Vec::new(),
        }
    }

    pub fn notify_all(&mut self, message: Message) {
        for client_stream in self.unix_clients.iter_mut() {
            if let Err(err) = client_stream.write_all(&message.as_bytes()) {
                warn!(
                    "failed to write message {:?} to client, err: {}",
                    message, err
                );
            }
            trace!("message sent to unix stream client: {:?}", message);
        }
    }

    pub fn add_client(&mut self, unix_stream: UnixStream) {
        self.unix_clients.push(unix_stream);
    }

    pub fn find_client_by_fd(&mut self, fd: u64) -> Option<&mut UnixStream> {
        self.unix_clients
            .iter_mut()
            .find(|stream| stream.as_raw_fd() as u64 == fd)
    }
}
