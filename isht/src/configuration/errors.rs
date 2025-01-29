use crate::ConfigStatment;

#[derive(Debug)]
pub enum ConfigurationError {
    NotRecognizedGroup(String),
    NotRecognizedKeybindMode(String),
    ExpectedTasksBlock,
    InvalidStatment(ConfigStatment),
}
impl std::error::Error for ConfigurationError {}
impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRecognizedGroup(name) => write!(
                f,
                "Ishtar does not recognize the name '{name}' for group configurations"
            ),
            Self::ExpectedTasksBlock => write!(f, "Expected Block for handling tasks"),
            Self::InvalidStatment(stmt) => write!(
                f,
                "Could not complete due to the statment '{stmt:?}' be in an invalid position"
            ),
            Self::NotRecognizedKeybindMode(mode) => write!(
                f,
                "Ishtar does not recognize a keyboard mode named as '{mode}"
            ),
        }
    }
}
