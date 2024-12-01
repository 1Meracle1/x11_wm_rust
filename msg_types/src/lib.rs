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
    FocusUp,
    FocusDown,
    MoveLeft,
    MoveRight,
    WorkspaceChange(u16),
    WorkspaceWindowChange(u16),
    WindowWidthGrow(u16),
    WindowWidthShrink(u16),
    WindowHeightGrow(u16),
    WindowHeightShrink(u16),
}

impl WmCommand {
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        match bincode::serialize(self) {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(format!("Serialize error: {:?}", err)),
        }
    }

    pub fn deserialize(bytes: &[u8]) -> Result<WmCommand, String> {
        match bincode::deserialize(bytes) {
            Ok(command) => Ok(command),
            Err(err) => Err(format!("Deserialize error: {:?}", err)),
        }
    }
}

impl TryFrom<WmMessage> for WmCommand {
    type Error = String;

    fn try_from(value: WmMessage) -> Result<Self, Self::Error> {
        if !value.parts.is_empty() {
            match value.parts.first().unwrap().as_str() {
                "focus" => {
                    if value.parts.len() == 2 {
                        match value.parts.get(1).unwrap().as_str() {
                            "left" => Ok(Self::FocusLeft),
                            "right" => Ok(Self::FocusRight),
                            "up" => Ok(Self::FocusUp),
                            "down" => Ok(Self::FocusDown),
                            _ => Err(format!("Invalid focus command: {}", value.parts.join(" "))),
                        }
                    } else {
                        Err(format!("Invalid focus command: {}", value.parts.join(" ")))
                    }
                }
                "move" => {
                    if value.parts.len() == 2 {
                        match value.parts.get(1).unwrap().as_str() {
                            "left" => Ok(Self::MoveLeft),
                            "right" => Ok(Self::MoveRight),
                            _ => Err(format!("Invalid move command: {}", value.parts.join(" "))),
                        }
                    } else {
                        Err(format!("Invalid move command: {}", value.parts.join(" ")))
                    }
                }
                "workspace" => {
                    if value.parts.len() == 3 && value.parts.get(1).unwrap() == "change" {
                        match value.parts.get(2).unwrap().as_str() {
                            new_workspace_id => match u16::from_str_radix(&new_workspace_id, 10) {
                                Ok(workspace_id) => Ok(Self::WorkspaceChange(workspace_id)),
                                Err(err) => Err(format!(
                                    "Invalid workspace change command: {}, error while extracting workspace id: {}",
                                    value.parts.join(" "),
                                    err
                                )),
                            },
                        }
                    } else if value.parts.len() == 4
                        && value.parts.get(1).unwrap() == "window"
                        && value.parts.get(2).unwrap() == "change"
                    {
                        match value.parts.get(3).unwrap().as_str() {
                            new_workspace_id => match u16::from_str_radix(&new_workspace_id, 10) {
                                Ok(workspace_id) => Ok(Self::WorkspaceWindowChange(workspace_id)),
                                Err(err) => Err(format!(
                                    "Invalid workspace window change command: {}, error while extracting workspace id: {}",
                                    value.parts.join(" "),
                                    err
                                )),
                            },
                        }
                    } else {
                        Err(format!(
                            "Invalid workspace change command: {}",
                            value.parts.join(" "),
                        ))
                    }
                }
                "grow" => {
                    if value.parts.len() == 3 {
                        let pixels_str = value.parts.get(2).unwrap();
                        let pixels: u16;
                        match u16::from_str_radix(&pixels_str, 10) {
                            Ok(res) => pixels = res,
                            Err(err) => {
                                return Err(format!(
                                    "Invalid value in place of number of pixel: {}, err: {}",
                                    pixels_str, err
                                ));
                            }
                        };
                        match value.parts.get(1).unwrap().as_str() {
                            "width" => Ok(Self::WindowWidthGrow(pixels)),
                            "height" => Ok(Self::WindowHeightGrow(pixels)),
                            val => Err(format!(
                                "Invalid dimension for window to be grown in: {}",
                                val
                            )),
                        }
                    } else {
                        Err(format!(
                            "Invalid window grow command: {}",
                            value.parts.join(" ")
                        ))
                    }
                }
                "shrink" => {
                    if value.parts.len() == 3 {
                        let pixels_str = value.parts.get(2).unwrap();
                        let pixels: u16;
                        match u16::from_str_radix(&pixels_str, 10) {
                            Ok(res) => pixels = res,
                            Err(err) => {
                                return Err(format!(
                                    "Invalid value in place of number of pixel: {}, err: {}",
                                    pixels_str, err
                                ));
                            }
                        };
                        match value.parts.get(1).unwrap().as_str() {
                            "width" => Ok(Self::WindowWidthShrink(pixels)),
                            "height" => Ok(Self::WindowHeightShrink(pixels)),
                            val => Err(format!(
                                "Invalid dimension for window to be shrinked in: {}",
                                val
                            )),
                        }
                    } else {
                        Err(format!(
                            "Invalid window shrink command: {}",
                            value.parts.join(" ")
                        ))
                    }
                }
                _ => Err(format!("Unknown command: {}", value.parts.join(" "))),
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
