use std::collections::VecDeque;

use anyhow::Result;

#[derive(Debug, Clone)]
pub enum CmdTask {
    //position
    SavePos,
    MoveSaved,
    //window
    CreateWindow,
    DeleteWindow,
    SetWindowUp,   //goes to the window above
    SetWindowDown, //goes to the window below
    //clipboard
    CopyToSys,
    CopyToEditor,
    PasteSys,
    PasteEditor,
    //fs
    CreateFile,
    RenameFile,
    DeleteFile,
    WriteOnFile,
    SaveFile,
    CopyFile,
    GetFileContent,
    Write(String), //writes on the current buffer; SysClip and EditorClip are reserved words for
    //System and Editor clipboard respectively
    //Cmd mode
    EnterNormal,
    EnterModify,
    EnterSelection, //mode to select
    //Cmds
    ExecCmd(String), //executes the following commands, its terminal ones.
    ExecutePrompt(String),
    //Selection
    SelectLine,
    DeleteLine,
    CopySelection,
    DeleteSelection,
    //Move
    MoveToLine(u32),
    MoveToRow(u32),
    MoveIOW, //init of word
    MoveEOW, //end of word
    MoveEOL, //end of line
    MoveIOL, //init of line
    MoveEOB, //end of buffer
    MoveIOB, // init of buffer

    Swap, //swap file buffers
}
impl CmdTask {
    pub fn new<S: Into<String>>(value: S) -> Result<Self, String> {
        let value = value.into();
        Ok(match value.as_ref() {
            "SavePos" => Self::SavePos,
            "MoveSaved" => Self::MoveSaved,
            "CreateWindow" => Self::CreateWindow,
            "DeleteWindow" => Self::DeleteWindow,
            "SetWindowUp" => Self::SetWindowUp,
            "SetWindowDown" => Self::SetWindowDown,
            "CopyToSys" => Self::CopyToSys,
            "CopyToEditor" => Self::CopyToEditor,
            "PasteSys" => Self::PasteSys,
            "PasteEditor" => Self::PasteEditor,
            "CreateFile" => Self::CreateFile,
            "RenameFile" => Self::RenameFile,
            "SaveFile" => Self::SaveFile,
            "DeleteFile" => Self::DeleteFile,
            "WriteOnFile" => Self::WriteOnFile,
            "CopyFile" => Self::CopyFile,
            "GetFileContent" => Self::GetFileContent,
            //Cmd mode
            "EnterCmd" | "EnterNormal" => Self::EnterNormal,
            "EnterModify" => Self::EnterModify,
            "EnterSelection" => Self::EnterSelection, //Mode to select
            //Selection
            "SelectLine" => Self::SelectLine,
            "DeleteLine" => Self::DeleteLine,
            "CopySelection" => Self::CopySelection,
            "DeleteSelection" => Self::DeleteSelection,
            //Move
            "MoveIOW" => Self::MoveIOW, //Init of word
            "MoveEOW" => Self::MoveEOW, //End of word
            "MoveEOL" => Self::MoveEOL, //End of line
            "MoveIOL" => Self::MoveIOL, //Init of line
            "MoveEOB" => Self::MoveEOB, //End of buffer
            "MoveIOB" => Self::MoveIOB, //Init of buffer

            "Swap" => Self::Swap, //Swap file buffers
            _ => return Err("Not known task named {value}".to_string()),
        })
    }
}
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
    Cmd(String),
}
#[derive(Debug)]
pub enum ConfigToken {
    Group(String),
    SubGroup(String),
    OpenBrace,
    CloseBrace,
    Task(CmdTask),
    Identifier(String),
    Arrow,
    Num(u32),
    String(String),
    Eq,
}
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

fn lex_content(content: String) -> VecDeque<ConfigToken> {
    let mut tokens = VecDeque::new();
    for (line_idx, line) in content.lines().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let mut idx = 0;
        while let Some(c) = chars.get(idx) {
            match c {
                '=' => tokens.push_back(ConfigToken::Eq),
                '"' => {
                    let mut buf = String::new();
                    idx += 1;
                    while let Some(c) = chars.get(idx) {
                        if *c == '"' {
                            break;
                        }
                        buf.push(*c);
                        idx += 1;
                    }
                    tokens.push_back(ConfigToken::String(buf));
                }
                '0'..='9' => {
                    let mut buf = String::new();
                    while let Some(c) = chars.get(idx) {
                        if matches!(c, '0'..='9') {
                            buf.push(*c);
                        } else {
                            break;
                        }
                        idx -= 1;
                        tokens.push_back(ConfigToken::Num(buf.parse::<u32>().unwrap()))
                    }
                }
                '-' => {
                    idx += 1;
                    if let Some('>') = chars.get(idx) {
                        tokens.push_back(ConfigToken::Arrow);
                    } else {
                        panic!("Was expecting '>' to make a '->' token at line {line_idx} and column {idx}");
                    }
                }
                '{' => tokens.push_back(ConfigToken::OpenBrace),
                '}' => tokens.push_back(ConfigToken::CloseBrace),
                '[' => {
                    idx += 1;
                    let mut buf = String::new();
                    while let Some(c) = chars.get(idx) {
                        idx += 1;
                        if *c == ']' {
                            break;
                        }
                        buf.push(*c);
                    }
                    tokens.push_back(ConfigToken::Group(buf));
                    idx -= 1;
                }
                '<' => {
                    idx += 1;
                    let mut buf = String::new();
                    while let Some(c) = chars.get(idx) {
                        idx += 1;
                        if *c == '>' {
                            break;
                        }
                        buf.push(*c);
                    }
                    tokens.push_back(ConfigToken::SubGroup(buf));
                    idx -= 1;
                }
                c => {
                    if c.is_whitespace() {
                        idx += 1;
                        continue;
                    }
                    let mut buf = String::new();
                    while let Some(c) = chars.get(idx) {
                        idx += 1;
                        if c.is_whitespace() {
                            break;
                        }
                        buf.push(*c);
                    }
                    if let Ok(task) = CmdTask::new(&buf) {
                        tokens.push_back(ConfigToken::Task(task));
                    } else {
                        tokens.push_back(ConfigToken::Identifier(buf))
                    }
                    idx -= 1;
                }
            }
            idx += 1;
        }
    }
    tokens
}
fn parse_block(tokens: &mut VecDeque<ConfigToken>) -> Result<ConfigStatment> {
    let mut block = Vec::new();
    loop {
        if let Some(ConfigToken::CloseBrace) = tokens.front() {
            tokens.pop_front();
            break;
        }
        match tokens.pop_front() {
            Some(ConfigToken::Identifier(s)) => {
                let task = match s.as_ref() {
                    "ExecCmd" | "ExecPrompt" | "Write" => {
                        if !matches!(tokens.pop_front(), Some(ConfigToken::Arrow)) {
                            return Err(IshtParseError::ExpectingArrow(s).into());
                        };
                        let Some(ConfigToken::String(content)) = tokens.pop_front() else {
                            return Err(IshtParseError::MissingStringParameter.into());
                        };
                        let task = match s.as_ref() {
                            "ExecCmd" => CmdTask::ExecCmd(content),
                            "ExecPrompt" => CmdTask::ExecutePrompt(content),
                            "Write" => CmdTask::Write(content),
                            _ => unreachable!(),
                        };
                        ConfigStatment::Task(task)
                    }
                    "MoveToLine" | "MoveToRow" => {
                        if !matches!(tokens.pop_front(), Some(ConfigToken::Arrow)) {
                            return Err(IshtParseError::ExpectingArrow(s).into());
                        };
                        let Some(ConfigToken::Num(n)) = tokens.pop_front() else {
                            return Err(IshtParseError::MissingNumParameter.into());
                        };
                        let task = match s.as_ref() {
                            "MoveToLine" => CmdTask::MoveToLine(n),
                            "MoveToRow" => CmdTask::MoveToRow(n),
                            _ => unreachable!(),
                        };
                        ConfigStatment::Task(task)
                    }
                    _ => {
                        if let Ok(task) = CmdTask::new(&s) {
                            ConfigStatment::Task(task)
                        } else {
                            ConfigStatment::Cmd(s)
                        }
                    }
                };
                block.push(task);
            }
            Some(ConfigToken::Task(task)) => block.push(ConfigStatment::Task(task)),
            Some(ConfigToken::OpenBrace) => block.push(parse_block(tokens)?),
            None => return Err(IshtParseError::ReachedEOF.into()),
            t => {
                return Err(
                    IshtParseError::InvalidTokenPosition(t.unwrap_or(ConfigToken::Arrow)).into(),
                )
            }
        }
    }
    Ok(ConfigStatment::Block(block))
}
fn parse_subgroup(
    subgroup_name: String,
    tokens: &mut VecDeque<ConfigToken>,
) -> Result<ConfigStatment> {
    if tokens.is_empty() {
        return Ok(ConfigStatment::SubGroup {
            name: subgroup_name,
            data: Box::new(ConfigStatment::Block(Vec::new())),
        });
    }
    let mut data = Vec::new();
    loop {
        let tk = tokens.front();
        if matches!(
            tk,
            Some(ConfigToken::Group(_) | ConfigToken::SubGroup(_)) | None
        ) {
            break;
        }
        if let Some(ConfigToken::Identifier(_)) = tk {
            let Some(ConfigToken::Identifier(tk)) = tokens.pop_front() else {
                unreachable!();
            };
            if matches!(
                tk.as_ref(),
                "ExecCmd" | "ExecPrompt" | "MoveToLine" | "MoveToRow" | "Write"
            ) {
                return Err(IshtParseError::WrongUseofReserved.into());
            }
            if let Some(ConfigToken::Eq) = tokens.pop_front() {
                let rhs = match tokens.pop_front() {
                    Some(ConfigToken::OpenBrace) => parse_block(tokens)?,
                    Some(ConfigToken::Identifier(s)) => match s.as_ref() {
                        "ExecCmd" | "ExecPrompt" | "Write" => {
                            if !matches!(tokens.pop_front(), Some(ConfigToken::Arrow)) {
                                return Err(IshtParseError::ExpectingArrow(s).into());
                            };
                            let Some(ConfigToken::String(content)) = tokens.pop_front() else {
                                return Err(IshtParseError::MissingStringParameter.into());
                            };
                            let task = match s.as_ref() {
                                "ExecCmd" => CmdTask::ExecCmd(content),
                                "ExecPrompt" => CmdTask::ExecutePrompt(content),
                                "Write" => CmdTask::Write(content),
                                _ => unreachable!(),
                            };
                            ConfigStatment::Task(task)
                        }
                        "MoveToLine" | "MoveToRow" => {
                            if !matches!(tokens.pop_front(), Some(ConfigToken::Arrow)) {
                                return Err(IshtParseError::ExpectingArrow(s).into());
                            };
                            let Some(ConfigToken::Num(n)) = tokens.pop_front() else {
                                return Err(IshtParseError::MissingNumParameter.into());
                            };
                            let task = match s.as_ref() {
                                "MoveToLine" => CmdTask::MoveToLine(n),
                                "MoveToRow" => CmdTask::MoveToRow(n),
                                _ => unreachable!(),
                            };
                            ConfigStatment::Task(task)
                        }
                        _ => {
                            if let Ok(task) = CmdTask::new(&s) {
                                ConfigStatment::Task(task)
                            } else {
                                ConfigStatment::Cmd(s)
                            }
                        }
                    },
                    Some(ConfigToken::Task(t)) => ConfigStatment::Task(t),
                    None => panic!("Reached EOF"),
                    t => {
                        return Err(IshtParseError::InvalidTokenPosition(
                            t.unwrap_or(ConfigToken::Arrow),
                        )
                        .into())
                    }
                };
                data.push(ConfigStatment::CmdDecl {
                    lhs: tk,
                    rhs: Box::new(rhs),
                });
            } else {
                return Err(IshtParseError::ExpectingEq.into());
            }
        } else {
            //arrow is used as default if not encountered any more tokens
            return Err(if tokens.is_empty() {
                IshtParseError::ReachedEOF.into()
            } else {
                IshtParseError::InvalidTokenPosition(tokens.pop_front().unwrap()).into()
            });
        }
    }
    Ok(ConfigStatment::SubGroup {
        name: subgroup_name,
        data: Box::new(ConfigStatment::Block(data)),
    })
}
fn parse_group(token: ConfigToken, tokens: &mut VecDeque<ConfigToken>) -> Result<ConfigStatment> {
    let ConfigToken::Group(group_name) = token else {
        unreachable!();
    };
    let mut subgroups = Vec::new();
    while let Some(ConfigToken::SubGroup(subgroup)) = tokens.pop_front() {
        subgroups.push(parse_subgroup(subgroup, tokens)?);
    }
    Ok(ConfigStatment::Group {
        name: group_name,
        data: Box::new(ConfigStatment::Block(subgroups)),
    })
}
fn parse_tokens(mut tokens: VecDeque<ConfigToken>) -> Result<ConfigStatment> {
    let mut statments = Vec::new();
    while let Some(token) = tokens.pop_front() {
        if !matches!(token, ConfigToken::Group(_)) {
            return Err(IshtParseError::BlockExpected.into());
        }
        statments.push(parse_group(token, &mut tokens)?);
    }
    Ok(ConfigStatment::Block(statments))
}
pub fn parse_content(content: String) -> Result<ConfigStatment> {
    let tokens = lex_content(content);
    parse_tokens(tokens)
}
