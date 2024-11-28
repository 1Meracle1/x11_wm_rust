use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct WmMessage {
    pub parts: Vec<String>,
}

impl WmMessage {
    pub fn new(parts: Vec<String>) -> Self {
        Self { parts }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum WmCommand {
    FocusLeft,
    FocusRight,
}

impl WmCommand {
    pub fn serialize(&self) -> Option<Vec<u8>> {
        if let Ok(bytes) = bincode::serialize(self) {
            Some(bytes)
        } else {
            None
        }
    }

    pub fn deserialize(bytes: &[u8]) -> Option<WmCommand> {
        if let Ok(command) = bincode::deserialize(bytes) {
            Some(command)
        } else {
            None
        }
    }
}

impl TryFrom<WmMessage> for WmCommand {
    type Error = String;

    fn try_from(value: WmMessage) -> Result<Self, Self::Error> {
        if !value.parts.is_empty() {
            if value.parts.first().unwrap().starts_with("focus") && value.parts.len() == 2 {
                let focus_type = value.parts.get(1).unwrap();
                if focus_type.starts_with("left") {
                    Ok(WmCommand::FocusLeft)
                } else if focus_type.starts_with("right") {
                    Ok(WmCommand::FocusRight)
                } else {
                    Err(format!("Invalid focus command: {}", value.parts.join(" ")))
                }
            } else {
                Err(format!("Unknown command: {}", value.parts.join(" ")))
            }
        } else {
            Err("Command should not be empty".to_string())
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
