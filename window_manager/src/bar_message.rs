use std::{io::Write, os::unix::net::UnixStream, path::Path};

use log::{trace, warn};

const MESSAGE_KEYBOARD_LAYOUT_TAG: u8 = 0;
const MESSAGE_WORKSPACE_LIST_TAG: u8 = 1;
const MESSAGE_WORKSPACE_ACTIVE_TAG: u8 = 2;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message<'a> {
    KeyboardLayout(&'a str),
    WorkspaceList(Vec<u32>),
    WorkspaceActive(u32),
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
        };

        // write actual size value in the first 8 bytes
        let message_size = (bytes.len() - 8) as u64;
        bytes[0..8].copy_from_slice(&message_size.to_le_bytes());

        bytes
    }
}

pub struct BarCommsBus<'a> {
    bar_socket_path: &'a Path,
    bar_unix_stream_maybe: Option<UnixStream>,
}

impl<'a> BarCommsBus<'a> {
    pub fn new(bar_socket_path: &'a Path) -> Self {
        if !bar_socket_path.exists() {
            warn!("no bar has unix listener setup");
        }
        BarCommsBus {
            bar_socket_path,
            bar_unix_stream_maybe: UnixStream::connect(bar_socket_path).ok(),
        }
    }

    pub fn send_message(&mut self, message: Message) {
        if self.bar_unix_stream_maybe.is_none() {
            trace!("unix stream to bar is not established, reconnecting");
            self.bar_unix_stream_maybe = UnixStream::connect(self.bar_socket_path).ok();
        }
        if let Some(bar_unix_stream) = &mut self.bar_unix_stream_maybe {
            if let Err(err) = bar_unix_stream.write_all(&message.as_bytes()) {
                warn!("failed to send message to the bar: {}", err);
            } else {
                if let Err(err) = bar_unix_stream.flush() {
                    warn!("failed to flush message to the bar: {}", err);
                } else {
                    trace!("sent notification to bar over unix stream: {:?}", message);
                }
            }
        } else {
            trace!("unix stream to bar is not established, no notification is sent");
        }
    }
}

// #[derive(Debug, PartialEq, Eq)]
// pub enum MessageDecodeError {
//     InputTooShort,
//     InvalidSize,
//     InvalidUtf8(std::string::FromUtf8Error),
//     InvalidTag,
//     IncorrectDataSize,
// }

// impl Message {
//     pub fn from_bytes(bytes: &[u8]) -> Result<Message, MessageDecodeError> {
//         if bytes.len() < 9 {

//         }

//     }
// }
