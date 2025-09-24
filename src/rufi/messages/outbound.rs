use alloc::collections::BTreeMap;
use core::hash::Hash;
use serde::Serialize;
use serde_value::Value;
use crate::rufi::messages::path::Path;

#[derive(Debug)]
pub struct OutboundMessage<Id: Ord + Hash + Copy> {
    pub sender: Id,
    underlying: BTreeMap<Path, Value>
}
impl <Id: Ord + Hash + Copy> OutboundMessage<Id> {
    pub fn empty(sender: Id) -> Self {
        Self { sender, underlying: BTreeMap::new() }
    }

    pub fn append<V>(&mut self, path: Path, value: V)
    where
        V: Serialize,
    {
        match serde_value::to_value(value) {
            Ok(serialized_value) => {
                self.underlying.insert(path, serialized_value);
            },
            Err(err) => panic!("Failed to serialize value: {:?}", err),
        }
    }
}
//     pub sender: Id,
//     underlying: BTreeMap<Path, Box<dyn Any>>,
// }
// impl<Id: Ord + Hash + Copy> OutboundMessage<Id> {
//     pub fn empty(sender: Id) -> Self {
//         Self {
//             sender,
//             underlying: BTreeMap::new(),
//         }
//     }
//
//     pub fn append<V: Any>(&mut self, path: Path, value: V)
//     {
//         self.underlying.insert(path, Box::new(value));
//     }
//
//     pub fn get<V: Any>(&self, path: &Path) -> Option<&V>
//     {
//         self.underlying
//             .get(path)
//             .and_then(|value| value.downcast_ref::<V>())
//     }
// }


