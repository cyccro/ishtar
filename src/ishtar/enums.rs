//used in mod.rs
#[derive(Debug, Clone)]
pub enum IshtarMode {
    Cmd,
    Modify,
}
#[derive(Debug)]
pub enum IshtarMessage {
    Null,
    ChangeMode(IshtarMode),
    Cmd(CmdResponse),
}
#[derive(Debug, Clone)]
pub enum CmdResponse {
    Exit,
    ChangeMode(IshtarMode),
    ModifyFile(String),
    Reset,
    SaveFile,
}
