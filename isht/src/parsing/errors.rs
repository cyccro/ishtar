use crate::lexer::ConfigToken;

#[derive(Debug)]
pub enum IshtParseError {
    WrongUseofReserved,
    ExpectingArrow(String),
    MissingStringParameter,
    MissingNumParameter,
    InvalidTokenPosition(ConfigToken),
    ExpectingEq,
    BlockExpected,
    ReachedEOF,
}
impl std::error::Error for IshtParseError {}
impl std::fmt::Display for IshtParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongUseofReserved => write!(f, "Using a reserved keyword in invalid position"),
            Self::ExpectingArrow(n) => write!(f, "Task '{n}' expects an arrow and an argument"),
            Self::MissingStringParameter => write!(f, "Received arrow but missing a string param"),
            Self::MissingNumParameter => write!(f, "Received arrow but missing a number parameter"),
            Self::InvalidTokenPosition(tk) => {
                write!(f, "Token {tk:?} is being used in invalid position")
            }
            Self::ExpectingEq => write!(
                f,
                "Expected an '=' for matching <Identifier> = <Tasks> rule"
            ),
            Self::BlockExpected => write!(f, "Expected a block statment"),
            Self::ReachedEOF => write!(f, "Reached EOF"),
        }
    }
}
