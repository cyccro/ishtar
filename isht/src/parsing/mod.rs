pub mod errors;
pub mod statment;
use std::collections::VecDeque;

use crate::{
    lexer::{lex_content, ConfigToken},
    tasks::CmdTask,
};
use anyhow::Result;
use errors::IshtParseError;
use statment::ConfigStatment;
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
