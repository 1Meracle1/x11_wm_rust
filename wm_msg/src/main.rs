use std::{io::Write, os::unix::net::UnixStream};

use msg_types::{WmCommand, WmMessage};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // println!("args: {:#?}", args);
    if args.is_empty() {
        println!("wm_msg v0.1");
        println!("wm_msg --help");
        return;
    }

    if let Ok(mut unix_stream) = UnixStream::connect("/tmp/x11_wm_rust") {
        let wm_message = WmMessage::new(args);
        match WmCommand::try_from(wm_message.clone()) {
            Ok(wm_command) => {
                match wm_command.serialize() {
                    Ok(bytes) => {
                        if let Err(err) = unix_stream.write(bytes.as_slice()) {
                            eprintln!("Failed to send message to the unix socket; error: {}", err);
                        }
                    }
                    Err(err) => eprintln!(
                        "Failed to serialize message to send to the unix socket; error: {}",
                        err
                    ),
                };
            }
            Err(error) => eprintln!("Error converting WmMessage to WmCommand: {}", error),
        }
    } else {
        eprintln!("Failed to connect to unix socket of the window manager.");
        return;
    }
}
