use anyhow::{Result, Error};
use lazy_static::lazy_static;
use std::{collections::HashMap, sync::{Arc, RwLock}};

use crate::{block::Block, identifier::Identifier};

#[derive(Default)]
pub struct Registry {
    blocks: HashMap<String, Vec<Block>>
}

impl Registry {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn current() -> Arc<Registry> {
        CURRENT_REGISTRY.read().unwrap().clone()
    }

    pub fn make_current(self) {
        *CURRENT_REGISTRY.write().unwrap() = Arc::new(self)
    }
}

lazy_static! {
    static ref CURRENT_REGISTRY: RwLock<Arc<Registry>> = RwLock::new(Arc::new(Registry::new()));
}

// registers
impl Registry {
    pub fn register_block(&mut self, block: Block) -> Result<(), Error> {
        match block.get_identifier().validate() {
            Ok(()) => {
                self.blocks.entry(block.get_identifier().get_namespace())
                    .or_insert(Vec::new())
                    .push(block);
                    
                Ok(())
            },
            Err(err) => Err(err)
        }
    }
}

// getters
impl Registry {
    pub fn get_block(&self, id: &Identifier) -> Option<Block> {
        match self.blocks.get(id.get_namespace().as_str()) {
            Some(blocks) => {
                match blocks.iter().find(|block| block.get_identifier().get_name() == id.get_name()) {
                    Some(block) => {
                        Some(block.clone())
                    },
                    None => None
                }
            },
            None => None
        }
    }
}