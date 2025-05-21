use std::{any::Any, collections::{HashMap, HashSet}, hash::Hash};

pub struct InboundMessage<Id: Ord + Hash + Copy> {
    underlying: HashMap<Id, ValueTree>,
}
impl <Id: Ord + Hash + Copy> InboundMessage<Id> {
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
            .filter_map(|(id, value_tree)| {
                value_tree.get::<V>(path).map(|value| (*id, value))
            })
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
impl <Id: Ord + Hash> OutboundMessage<Id> {
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
