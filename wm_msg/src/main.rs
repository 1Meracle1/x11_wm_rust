use std::{io::Write, os::unix::net::UnixStream};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // println!("args: {:#?}", args);
    if args.is_empty() {
        println!("wm_msg v0.1");
        println!("wm_msg --help");
        return;
    }

    if let Ok(mut unix_stream) = UnixStream::connect("/tmp/x11_wm_rust") {
        if let Some(first_command_word) = args.first() {
            match first_command_word {
                // resize shrink width 1
                // resize grow width 1
                _ if first_command_word == "resize" => {
                    if args.len() >= 4 {
                        let msg = format!(
                            "{} {} {} {}",
                            first_command_word,
                            args.get(1).unwrap(),
                            args.get(2).unwrap(),
                            args.get(3).unwrap()
                        );
                        if let Err(err) = unix_stream.write(msg.as_bytes()) {
                            eprintln!("Failed to send {} to window manager; error: {}", msg, err);
                        }
                    } else {
                        eprintln!("invalid {} command", first_command_word);
                        return;
                    }
                }
                _ if first_command_word == "focus" => {
                    // focus left
                    // focus right
                    // focus up
                    // focus down
                    if args.len() >= 2 {
                        let msg = format!("{} {}", first_command_word, args.get(1).unwrap());
                        if let Err(err) = unix_stream.write(msg.as_bytes()) {
                            eprintln!("Failed to send {} to window manager; error: {}", msg, err);
                        }
                    } else {
                        eprintln!("invalid {} command", first_command_word);
                        return;
                    }
                }
                _ if first_command_word == "move" => {
                    // move left
                    // move right
                    // move up
                    // move down
                    if args.len() >= 2 {
                        let msg = format!("{} {}", first_command_word, args.get(1).unwrap());
                        if let Err(err) = unix_stream.write(msg.as_bytes()) {
                            eprintln!("Failed to send {} to window manager; error: {}", msg, err);
                        }
                    } else {
                        eprintln!("invalid {} command", first_command_word);
                        return;
                    }
                }
                _ => {
                    eprintln!("Unknown command: {}", first_command_word);
                }
            }
        }
    } else {
        eprintln!("Failed to connect to unix socket of the window manager.");
        return;
    }
}
