extern crate alloc;

use crate::rufi::messages::Path;
use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Display;
use core::fmt::Formatter;

#[derive(Debug, Clone)]
pub(crate) struct InvocationCoordinate {
    counter: u16,
    token: String,
}
impl InvocationCoordinate {
    pub(crate) fn new(counter: u16, token: String) -> Self {
        Self { counter, token }
    }
}
impl Display for InvocationCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.counter, self.token)
    }
}

pub(crate) struct AlignmentStack {
    stack: VecDeque<InvocationCoordinate>,
    trace: BTreeMap<Path, u16>,
}
impl AlignmentStack {
    pub(crate) fn new() -> Self {
        Self {
            stack: VecDeque::new(),
            trace: BTreeMap::new(),
        }
    }

    pub(crate) fn current_path(&self) -> Vec<InvocationCoordinate> {
        self.stack.iter().cloned().collect()
    }

    pub(crate) fn align(&mut self, token: String) {
        let current_path = Path::new(self.stack.iter().cloned().collect());
        let current_counter = self
            .trace
            .get(&current_path)
            .map(|counter| counter + 1)
            .unwrap_or(0);
        let invocation_coordinate = InvocationCoordinate::new(current_counter, token);
        self.stack.push_back(invocation_coordinate);
        self.trace.insert(current_path, current_counter);
    }

    pub(crate) fn unalign(&mut self) {
        self.stack.pop_back();
    }

    // pub(crate) fn align_on<F, R>(&mut self, token: String, body: F) -> R
    // where
    //     F: FnOnce() -> R,
    // {
    //     self.align(token);
    //     let result = body();
    //     self.unalign();
    //     result
    // }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::ToString;
    #[test]
    fn invocation_coordinate_display() {
        let invocation_coordinate = super::InvocationCoordinate::new(1, "test".to_string());
        assert_eq!(invocation_coordinate.to_string(), "1:test");
    }

    #[test]
    fn alignment_stack_simple() {
        let mut stack = super::AlignmentStack::new();
        stack.align("test".to_string());
        assert_eq!(stack.current_path().len(), 1);
        assert_eq!(stack.current_path()[0].token, "test");
        stack.unalign();
        assert_eq!(stack.current_path().len(), 0);
    }

    #[test]
    fn alignment_stack_nested() {
        let mut stack = super::AlignmentStack::new();
        stack.align("outer".to_string());
        stack.align("inner".to_string());
        assert_eq!(stack.current_path().len(), 2);
        assert_eq!(stack.current_path()[0].token, "outer");
        assert_eq!(stack.current_path()[1].token, "inner");
        stack.unalign();
        assert_eq!(stack.current_path().len(), 1);
        assert_eq!(stack.current_path()[0].token, "outer");
    }

    #[test]
    fn alignment_stack_same_token() {
        let mut stack = super::AlignmentStack::new();
        stack.align("test".to_string());
        assert_eq!(stack.current_path()[0].token, "test");
        assert_eq!(stack.current_path()[0].counter, 0);
        stack.unalign();

        stack.align("test".to_string());
        // print!("{:?}", stack.current_path()); // Removed for no_std compatibility
        assert_eq!(stack.current_path()[0].token, "test");
        assert_eq!(stack.current_path()[0].counter, 1);
        stack.unalign();
    }
}
