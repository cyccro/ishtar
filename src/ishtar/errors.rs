use std::fmt;

#[derive(Debug)]
pub enum IshtarError {}
impl std::error::Error for IshtarError {}
impl std::fmt::Display for IshtarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Must implement")
    }
}
