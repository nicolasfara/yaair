use crate::rufi::messages::path::Path;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::any::Any;

pub struct State {
    last_state: BTreeMap<Path, Box<dyn Any>>,
}
impl State {
    pub fn new() -> Self {
        Self {
            last_state: BTreeMap::new(),
        }
    }

    pub fn from_snapshot(snapshot: BTreeMap<Path, Box<dyn Any>>) -> Self {
        Self {
            last_state: snapshot,
        }
    }

    pub fn insert<V: Any>(&mut self, path: Path, value: V) {
        self.last_state.insert(path, Box::new(value));
    }

    pub fn get<V: Any>(&self, path: &Path) -> Option<&V> {
        match self.last_state.get(path) {
            Some(value) => match value.downcast_ref::<V>() {
                Some(v) => Some(v),
                None => panic!(
                    "Type mismatch in repeat state at path {:?}. \
                    Expected type {} but found different type in stored state. \\
                    This usually indicates the same alignment path is being used \
                    for different value types across iterations.",
                    path,
                    core::any::type_name::<V>()
                ),
            },
            None => None,
        }
    }
}
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rufi::messages::path::Path;
    use alloc::string::ToString;
    use alloc::vec;
    use core::any::Any;

    fn make_path(id: u32) -> Path {
        Path::new(vec![id.to_string()])
    }

    #[test]
    fn test_new_and_default() {
        let s1 = State::new();
        let s2 = State::default();
        assert_eq!(s1.last_state.len(), 0);
        assert_eq!(s2.last_state.len(), 0);
    }

    #[test]
    fn test_insert_and_get_success() {
        let mut state = State::new();
        let path = make_path(1);
        state.insert(path.clone(), 42u32);
        let value = state.get::<u32>(&path);
        assert_eq!(value, Some(&42u32));
    }

    #[test]
    #[should_panic(expected = "Type mismatch in repeat state")]
    fn test_get_type_mismatch_panics() {
        let mut state = State::new();
        let path = make_path(2);
        state.insert(path.clone(), 3.14f32);
        // Panic expected here due to type mismatch
        let _ = state.get::<u32>(&path);
    }

    #[test]
    fn test_get_none_for_missing_path() {
        let state = State::new();
        let path = make_path(3);
        assert_eq!(state.get::<u32>(&path), None);
    }

    #[test]
    fn test_from_snapshot() {
        let path = make_path(4);
        let mut snapshot = BTreeMap::new();
        snapshot.insert(path.clone(), Box::new(99u8) as Box<dyn Any>);
        let state = State::from_snapshot(snapshot);
        assert_eq!(state.get::<u8>(&path), Some(&99u8));
    }
}
