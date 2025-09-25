use crate::rufi::messages::path::Path;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::hash::Hash;

#[derive(Debug)]
pub struct OutboundMessage<Id: Ord + Hash + Copy> {
    pub sender: Id,
    underlying: BTreeMap<Path, Vec<u8>>,
}
impl<Id: Ord + Hash + Copy> OutboundMessage<Id> {
    pub fn empty(sender: Id) -> Self {
        Self {
            sender,
            underlying: BTreeMap::new(),
        }
    }

    pub fn append(&mut self, path: Path, value: Vec<u8>) {
        self.underlying.insert(path, value);
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
