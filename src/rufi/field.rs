use std::collections::HashMap;

#[derive(Debug)]
pub struct Field<D: Ord, V> {
    default: V,
    overrides: HashMap<D, V>,
}

impl<D: Ord, V> Field<D, V> {
    pub(crate) fn new(default: V, overrides: HashMap<D, V>) -> Field<D, V> {
        Field { default, overrides }
    }

    pub fn local(&self) -> &V {
        &self.default
    }
}
