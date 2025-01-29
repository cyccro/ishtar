use crate::ConfigStatment;

#[derive(Debug)]
pub enum ConfigurationError {
    NotRecognizedName(String),
    ExpectedTasksBlock,
    InvalidStatment(ConfigStatment),
}
impl std::error::Error for ConfigurationError {}
impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotRecognizedName(name) => write!(
                f,
                "Ishtar does not recognize the name {name} for configurations"
            ),
            Self::ExpectedTasksBlock => write!(f, "Expected Block for handling tasks"),
            Self::InvalidStatment(stmt) => write!(
                f,
                "Could not the statment {stmt:?} due to it being in an invalid position"
            ),
        }
    }
}
