use crate::yaair::messages::path::Path;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as Map;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::hash::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap as Map;

#[derive(Debug, Serialize, Deserialize)]
pub struct OutboundMessage<Id: Ord + Hash + Copy> {
    pub sender: Id,
    underlying: Map<String, Vec<u8>>,
}
impl<Id: Ord + Hash + Copy> OutboundMessage<Id> {
    pub fn empty(sender: Id) -> Self {
        Self {
            sender,
            underlying: Map::new(),
        }
    }

    pub fn append(&mut self, path: &Path, value: Vec<u8>) {
        self.underlying.insert(path.to_string(), value);
    }

    pub fn at(&self, path: &Path) -> Option<&Vec<u8>> {
        self.underlying.get(&path.to_string())
    }
}
