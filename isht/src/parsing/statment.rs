use crate::tasks::CmdTask;

#[derive(Debug, Clone)]
pub enum ConfigStatment {
    Group {
        name: String,
        data: Box<ConfigStatment>,
    },
    SubGroup {
        name: String,
        data: Box<ConfigStatment>,
    },
    CmdDecl {
        lhs: String,
        rhs: Box<ConfigStatment>,
    },
    Block(Vec<ConfigStatment>),
    Task(CmdTask),
    Color(u32),
    Cmd(String),
}
