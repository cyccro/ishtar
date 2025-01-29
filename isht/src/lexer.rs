use std::collections::VecDeque;

use crate::tasks::CmdTask;

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
pub fn lex_content(content: String) -> VecDeque<ConfigToken> {
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
                        if c.is_ascii_digit() {
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
