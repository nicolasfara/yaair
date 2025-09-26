use alloc::string::{String, ToString};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Serialize, Deserialize)]
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
    use alloc::collections::BTreeSet;

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

        let mut set = BTreeSet::new();
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
}
