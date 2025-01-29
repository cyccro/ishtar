mod errors;
use crate::{parse_content, ConfigStatment};
use anyhow::Result;
use errors::ConfigurationError;
use std::collections::HashMap;

type Keybinds = HashMap<String, Vec<ConfigStatment>>;

#[derive(Debug)]
pub struct IshtarConfiguration {
    pub keybinds: [Keybinds; 3], //normal, modify, selection
}

impl IshtarConfiguration {
    ///Generates data into the target based on the keybind group
    pub fn generate_from_keybinds_group(
        data: &ConfigStatment,
        target: &mut [Keybinds; 3],
    ) -> Result<()> {
        let ConfigStatment::Block(contents) = &data else {
            unreachable!();
        };
        for content in contents {
            let ConfigStatment::SubGroup { name, data } = content else {
                unreachable!();
            };
            let keybind = match name.as_ref() {
                "normal" => &mut target[0],
                "modify" => &mut target[1],
                "selection" => &mut target[2],
                _ => return Err(ConfigurationError::NotRecognizedKeybindMode(name.clone()).into()),
            };
            let ConfigStatment::Block(data) = &**data else {
                return Err(ConfigurationError::ExpectedTasksBlock.into());
            };
            for data in data.iter() {
                match data {
                    //Sadly, i prefer doing this than sending data between threads with Arc. Im not
                    //that good for handling this kind of stuff yet but probably imma do it later
                    ConfigStatment::CmdDecl { lhs, rhs } => {
                        let cmd = match &**rhs {
                            ConfigStatment::Block(cmds) => cmds.clone(),
                            c => vec![c.clone()],
                        };
                        keybind.insert(lhs.clone(), cmd);
                    }
                    e => return Err(ConfigurationError::InvalidStatment(e.clone()).into()),
                }
            }
        }
        Ok(())
    }
    ///Sets a configuration based in the given content string. It's supposed to be a valid Isht
    ///content
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
                            Self::generate_from_keybinds_group(&data, &mut this.keybinds)?
                        }
                        _ => {
                            return Err(ConfigurationError::NotRecognizedGroup(name.clone()).into())
                        }
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
