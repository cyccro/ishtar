//used in mod.rs
#[derive(Debug, Clone)]
pub enum IshtarMode {
    Cmd,
    Modify,
    Selection,
}
//A message from the terminal
#[derive(Debug)]
pub enum IshtarMessage {
    Null,
    ChangeMode(IshtarMode),
    Cmd(CmdResponse),
}
//A response from cmd interpreter
#[derive(Debug, Clone)]
pub enum CmdResponse {
    Exit,
    ChangeMode(IshtarMode),
    ModifyFile(String),
    Reset,
    SaveFile,
}
