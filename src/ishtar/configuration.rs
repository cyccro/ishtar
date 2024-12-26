use anyhow::Result;
use isht::{parse_content, ConfigStatment};

use std::collections::HashMap;

#[derive(Debug)]
pub struct IshtarConfiguration {
    pub keybinds: [HashMap<String, Vec<ConfigStatment>>; 3], //normal, modify, selection
}

impl IshtarConfiguration {
    pub fn from_content(content: String) -> Result<Self> {
        let statments = parse_content(content)?;
        let ConfigStatment::Block(contents) = statments else {
            unreachable!();
        };
        let mut this = Self::new();
        for content in contents.iter() {
            match content {
                ConfigStatment::Group { name, data } => {
                    let nameref = name.as_ref();
                    match nameref {
                        "keybinds" => {
                            let ConfigStatment::Block(contents) = &**data else {
                                unreachable!();
                            };
                            for content in contents {
                                let ConfigStatment::SubGroup { name, data } = content else {
                                    unreachable!();
                                };
                                let keybind = match name.as_ref() {
                                    "normal" => &mut this.keybinds[0],
                                    "modify" => &mut this.keybinds[1],
                                    "selection" => &mut this.keybinds[2],
                                    _ => panic!(
                                        "'{name}' subgroup is not supported in {nameref} group"
                                    ),
                                };
                                let ConfigStatment::Block(data) = &**data else {
                                    panic!("Should receive a block for handling N keybinds");
                                };
                                for data in data.iter() {
                                    match data {
                                        ConfigStatment::CmdDecl { lhs, rhs } => {
                                            let cmd = match &**rhs {
                                                ConfigStatment::Block(cmds) => cmds.clone(),
                                                c => vec![c.clone()],
                                            };
                                            keybind.insert(lhs.clone(), cmd);
                                        }
                                        e => {
                                            panic!("Wasnt expecting {e:?}. Instead a CmdDecl here")
                                        }
                                    }
                                }
                            }
                        }
                        _ => panic!("Ishtar does not handle data for '{name}' group"),
                    }
                }
                _ => unreachable!(),
            };
        }
        Ok(this)
    }
    pub fn new() -> Self {
        Self {
            keybinds: [HashMap::new(), HashMap::new(), HashMap::new()],
        }
    }
}
impl Default for IshtarConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
