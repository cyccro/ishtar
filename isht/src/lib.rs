pub mod configuration;
mod lexer;
mod parsing;
mod tasks;
pub use parsing::errors::IshtParseError;
pub use parsing::parse_content;
pub use parsing::statment::ConfigStatment;
pub use tasks::CmdTask;
