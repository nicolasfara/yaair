use crate::rufi::messages::path::Path;
use crate::rufi::messages::valuetree::ValueTree;
#[cfg(not(feature = "std"))]
use alloc::collections::{BTreeMap as Map, BTreeSet as Set};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::hash::Hash;
use std::collections::{HashMap as Map, HashSet as Set};

#[derive(Debug)]
pub struct InboundMessage<Id: Ord + Hash + Copy> {
    underlying: Map<Id, ValueTree>,
}
impl<Id: Ord + Hash + Copy> InboundMessage<Id> {
    pub const fn new(underlying: Map<Id, ValueTree>) -> Self {
        Self { underlying }
    }

    pub fn get(&self, id: &Id) -> Option<&ValueTree> {
        self.underlying.get(id)
    }

    pub fn get_at_path(&self, path: &Path) -> Map<Id, Vec<u8>> {
        self.underlying
            .iter()
            .filter_map(|(id, value_tree)| value_tree.get(path).map(|value| (*id, value)))
            .collect()
    }

    pub fn devices_at_path(&self, path: &Path) -> Set<Id> {
        self.underlying
            .iter()
            .filter_map(|(id, value_tree)| {
                if value_tree.contains_key(path) {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}
impl<Id: Ord + Hash + Copy> Default for InboundMessage<Id> {
    fn default() -> Self {
        Self {
            underlying: Map::new(),
        }
    }
}
