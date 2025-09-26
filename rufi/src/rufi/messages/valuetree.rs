use crate::rufi::messages::path::Path;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct ValueTree {
    underlying: BTreeMap<Path, Vec<u8>>,
}

impl ValueTree {
    pub const fn empty() -> Self {
        Self {
            underlying: BTreeMap::new(),
        }
    }

    pub const fn new(underlying: BTreeMap<Path, Vec<u8>>) -> Self {
        Self { underlying }
    }

    pub fn contains_key(&self, path: &Path) -> bool {
        self.underlying.contains_key(path)
    }

    pub fn get(&self, path: &Path) -> Option<Vec<u8>> {
        self.underlying.get(path).cloned()
    }

    // pub fn insert<T>(&mut self, path: Path, value: T)
    // where
    //     T: Serialize,
    // {
    //     match serde_value::to_value(value) {
    //         Ok(serialized_value) => {
    //             let _ = self.underlying.insert(path, serialized_value);
    //         }
    //         Err(err) => panic!("Failed to serialize value: {}", err),
    //     }
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use alloc::string::{String, ToString};
//
//     fn make_path(tokens: &[&str]) -> Path {
//         Path::new(tokens.to_vec())
//     }
//
//     #[test]
//     fn test_value_tree_get_and_new() {
//         let mut underlying = BTreeMap::new();
//         let path = make_path(&["x"]);
//         underlying.insert(path.clone(), serde_value::to_value(99u8).unwrap());
//         let vt = ValueTree::new(underlying);
//
//         assert_eq!(vt.get::<u8>(&path), Some(99u8));
//         assert_eq!(vt.get::<u8>(&make_path(&["y"])), None);
//         assert_eq!(vt.get::<i32>(&path), Some(99i32)); // Value conversion flexibility
//     }
//
//     #[test]
//     fn test_value_tree_empty() {
//         let vt = ValueTree::empty();
//         let path = make_path(&["z"]);
//         assert!(vt.get::<u8>(&path).is_none());
//     }
//
//     #[test]
//     fn test_value_tree_insert() {
//         let mut vt = ValueTree::empty();
//         let path = make_path(&["test"]);
//
//         vt.insert(path.clone(), "hello world");
//         assert_eq!(vt.get::<String>(&path), Some("hello world".to_string()));
//
//         vt.insert(path.clone(), 42i32);
//         assert_eq!(vt.get::<i32>(&path), Some(42));
//     }
// }
