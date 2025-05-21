use std::{
    any::Any,
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub struct InboundMessage<Id: Ord + Hash + Copy> {
    underlying: HashMap<Id, ValueTree>,
}
impl<Id: Ord + Hash + Copy> InboundMessage<Id> {
    pub fn new(underlying: HashMap<Id, ValueTree>) -> Self {
        Self { underlying }
    }

    pub fn get(&self, id: &Id) -> Option<&ValueTree> {
        self.underlying.get(id)
    }

    pub fn get_at_path<V>(&self, path: &Path) -> HashMap<Id, &V>
    where
        V: 'static,
    {
        self.underlying
            .iter()
            .filter_map(|(id, value_tree)| value_tree.get::<V>(path).map(|value| (*id, value)))
            .collect()
    }

    pub fn devices_at_path(&self, path: &Path) -> HashSet<Id> {
        self.underlying
            .iter()
            .filter_map(|(id, value_tree)| {
                if value_tree.get::<()>(path).is_some() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}
pub struct OutboundMessage<Id: Ord + Hash> {
    pub sender: Id,
    underlying: HashMap<Path, Box<dyn Any>>,
}
impl<Id: Ord + Hash> OutboundMessage<Id> {
    pub fn empty(sender: Id) -> Self {
        Self {
            sender,
            underlying: HashMap::new(),
        }
    }

    pub fn append<V>(&mut self, path: Path, value: V)
    where
        V: 'static,
    {
        self.underlying.insert(path, Box::new(value));
    }

    pub fn get<V>(&self, path: &Path) -> Option<&V>
    where
        V: 'static,
    {
        self.underlying
            .get(path)
            .and_then(|value| value.downcast_ref::<V>())
    }
}

pub struct ValueTree {
    underlying: HashMap<Path, Box<dyn Any>>,
}
impl ValueTree {
    pub fn empty() -> Self {
        Self {
            underlying: HashMap::new(),
        }
    }

    pub fn new(underlying: HashMap<Path, Box<dyn Any>>) -> Self {
        Self { underlying }
    }

    pub fn get<T: 'static>(&self, path: &Path) -> Option<&T> {
        self.underlying
            .get(path)
            .and_then(|value| value.downcast_ref::<T>())
    }
}

pub trait Exportable<Id: Ord + Hash> {
    fn export(&self) -> OutboundMessage<Id>;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub struct Path {
    tokens: Vec<String>,
}

impl Path {
    pub fn new<T: ToString>(tokens: Vec<T>) -> Self {
        Self {
            tokens: tokens.into_iter().map(|t| t.to_string()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn make_path(tokens: &[&str]) -> Path {
        Path::new(tokens.to_vec())
    }

    #[test]
    fn test_path_equality_and_hash() {
        let p1 = make_path(&["a", "b"]);
        let p2 = make_path(&["a", "b"]);
        let p3 = make_path(&["a", "c"]);
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);

        let mut set = HashSet::new();
        set.insert(p1);
        assert!(set.contains(&p2));
        assert!(!set.contains(&p3));
    }

    #[test]
    fn test_path_ordering() {
        let p1 = make_path(&["a"]);
        let p2 = make_path(&["a", "b"]);
        let p3 = make_path(&["b"]);
        assert!(p1 < p2);
        assert!(p2 < p3);
    }

    #[test]
    fn test_outbound_message_append_and_get() {
        let mut msg = OutboundMessage::empty(42u32);
        let path = make_path(&["foo"]);
        msg.append(path.clone(), 123i32);

        assert_eq!(msg.get::<i32>(&path), Some(&123));
        assert_eq!(msg.get::<i32>(&make_path(&["bar"])), None);
        assert_eq!(msg.sender, 42u32);
    }

    #[test]
    fn test_value_tree_get_and_new() {
        let mut underlying = HashMap::new();
        let path = make_path(&["x"]);
        underlying.insert(path.clone(), Box::new(99u8) as Box<dyn Any>);
        let vt = ValueTree::new(underlying);

        assert_eq!(vt.get::<u8>(&path), Some(&99u8));
        assert_eq!(vt.get::<u8>(&make_path(&["y"])), None);
        assert_eq!(vt.get::<i32>(&path), None);
    }

    #[test]
    fn test_value_tree_empty() {
        let vt = ValueTree::empty();
        let path = make_path(&["z"]);
        assert!(vt.get::<u8>(&path).is_none());
    }

    #[test]
    fn test_inbound_message_get_and_get_at_path() {
        let mut vt_map = HashMap::new();
        let path = make_path(&["foo"]);
        let mut vt1 = ValueTree::empty();
        vt1.underlying.insert(path.clone(), Box::new(1u32));
        let mut vt2 = ValueTree::empty();
        vt2.underlying.insert(path.clone(), Box::new(2u32));
        vt_map.insert(10u8, vt1);
        vt_map.insert(20u8, vt2);

        let inbound = InboundMessage::new(vt_map);

        // get
        assert!(inbound.get(&10u8).is_some());
        assert!(inbound.get(&30u8).is_none());

        // get_at_path
        let map = inbound.get_at_path::<u32>(&path);
        assert_eq!(map.len(), 2);
        assert_eq!(*map.get(&10u8).unwrap(), &1u32);
        assert_eq!(*map.get(&20u8).unwrap(), &2u32);

        // get_at_path with wrong type
        let map_wrong = inbound.get_at_path::<i32>(&path);
        assert!(map_wrong.is_empty());
    }

    #[test]
    fn test_inbound_message_devices_at_path() {
        let mut vt_map = HashMap::new();
        let path = make_path(&["foo"]);
        let mut vt1 = ValueTree::empty();
        vt1.underlying.insert(path.clone(), Box::new(()) as Box<dyn Any>);
        let mut vt2 = ValueTree::empty();
        vt2.underlying.insert(path.clone(), Box::new(()) as Box<dyn Any>);
        let vt3 = ValueTree::empty();
        // vt3 does not have the path

        vt_map.insert(1u8, vt1);
        vt_map.insert(2u8, vt2);
        vt_map.insert(3u8, vt3);

        let inbound = InboundMessage::new(vt_map);
        let devices = inbound.devices_at_path(&path);

        assert_eq!(devices.len(), 2);
        assert!(devices.contains(&1u8));
        assert!(devices.contains(&2u8));
        assert!(!devices.contains(&3u8));
    }
}