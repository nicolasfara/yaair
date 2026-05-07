use crate::yaair::messages::path::Path;
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as Map;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use core::hash::Hash;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "std")]
use std::collections::HashMap as Map;

#[derive(Debug)]
pub struct OutboundMessage<Id: Ord + Hash + Copy> {
    pub sender: Id,
    underlying: Map<Path, Vec<u8>>,
}
impl<Id: Ord + Hash + Copy> OutboundMessage<Id> {
    pub fn empty(sender: Id) -> Self {
        Self {
            sender,
            underlying: Map::new(),
        }
    }

    pub fn append(&mut self, path: &Path, value: Vec<u8>) {
        self.underlying.insert(path.clone(), value);
    }

    pub fn at(&self, path: &Path) -> Option<&Vec<u8>> {
        self.underlying.get(path)
    }
}

impl<Id> Serialize for OutboundMessage<Id>
where
    Id: Ord + Hash + Copy + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("OutboundMessage", 2)?;
        state.serialize_field("sender", &self.sender)?;
        let underlying = self.underlying.iter().collect::<Vec<_>>();
        state.serialize_field("underlying", &underlying)?;
        state.end()
    }
}

impl<'de, Id> Deserialize<'de> for OutboundMessage<Id>
where
    Id: Ord + Hash + Copy + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire<Id> {
            sender: Id,
            underlying: Vec<(Path, Vec<u8>)>,
        }

        let wire = Wire::<Id>::deserialize(deserializer)?;
        Ok(Self {
            sender: wire.sender,
            underlying: wire.underlying.into_iter().collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_should_preserve_tokens_containing_slashes() {
        let path = Path::new(vec!["outer/inner", "leaf"]);
        let mut outbound = OutboundMessage::empty(1u32);
        outbound.append(&path, vec![1, 2, 3]);

        let serialized = serde_json::to_vec(&outbound).unwrap();
        let deserialized: OutboundMessage<u32> = serde_json::from_slice(&serialized).unwrap();

        assert_eq!(deserialized.at(&path), Some(&vec![1, 2, 3]));
        assert_eq!(deserialized.at(&Path::from("outer/inner/leaf")), None);
    }

    #[test]
    fn serialization_should_preserve_empty_paths() {
        let path = Path::new(Vec::<&str>::new());
        let mut outbound = OutboundMessage::empty(1u32);
        outbound.append(&path, vec![4, 5, 6]);

        let serialized = serde_json::to_vec(&outbound).unwrap();
        let deserialized: OutboundMessage<u32> = serde_json::from_slice(&serialized).unwrap();

        assert_eq!(deserialized.at(&path), Some(&vec![4, 5, 6]));
        assert_eq!(deserialized.at(&Path::from("")), None);
    }
}
