use crate::rufi::messages::path::Path;
use alloc::collections::BTreeMap;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Display;
use core::fmt::Formatter;
use core::num::Saturating;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InvocationCoordinate {
    counter: u32,
    token: String,
}
impl InvocationCoordinate {
    pub(crate) fn new(counter: u32, token: impl Into<String>) -> Self {
        Self {
            counter,
            token: token.into(),
        }
    }
}
impl Display for InvocationCoordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.token, self.counter)
    }
}

pub(crate) struct AlignmentStack {
    stack: VecDeque<InvocationCoordinate>,
    trace: BTreeMap<Path, Saturating<u32>>,
}
impl AlignmentStack {
    pub(crate) const fn new() -> Self {
        Self {
            stack: VecDeque::new(),
            trace: BTreeMap::new(),
        }
    }

    pub(crate) fn current_path(&self) -> Vec<InvocationCoordinate> {
        self.stack.iter().cloned().collect()
    }

    pub(crate) fn align(&mut self, token: impl Into<String>) {
        let current_path = Path::new(self.stack.iter().cloned().collect());
        let current_counter = self
            .trace
            .get(&current_path)
            .map_or(Saturating(0), |counter| counter + Saturating(1));
        let invocation_coordinate = InvocationCoordinate::new(current_counter.0, token.into());
        self.stack.push_back(invocation_coordinate);
        self.trace.insert(current_path, current_counter);
    }

    pub(crate) fn unalign(&mut self) {
        self.stack.pop_back();
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use crate::rufi::alignment::alignment_stack::InvocationCoordinate;

    #[test]
    fn invocation_coordinate_display() {
        let invocation_coordinate = super::InvocationCoordinate::new(1, "test");
        assert_eq!(invocation_coordinate.to_string(), "test:1");
    }

    #[test]
    fn alignment_stack_simple() {
        let mut stack = super::AlignmentStack::new();
        stack.align("test");
        assert_eq!(stack.current_path().len(), 1);
        let expected = InvocationCoordinate::new(0, "test");
        assert_eq!(stack.current_path().first(), Some(&expected));
        stack.unalign();
        assert_eq!(stack.current_path().len(), 0);
    }

    #[test]
    fn alignment_stack_nested() {
        let mut stack = super::AlignmentStack::new();
        stack.align("outer");
        stack.align("inner");
        assert_eq!(stack.current_path().len(), 2);
        let expected_outer = InvocationCoordinate::new(0, "outer");
        let expected_inner = InvocationCoordinate::new(0, "inner");
        assert_eq!(stack.current_path().first(), Some(&expected_outer));
        assert_eq!(stack.current_path().get(1), Some(&expected_inner));
        stack.unalign();
        assert_eq!(stack.current_path().len(), 1);
        assert_eq!(stack.current_path().first(), Some(&expected_outer));
    }

    #[test]
    fn alignment_stack_same_token() {
        let mut stack = super::AlignmentStack::new();
        stack.align("test");
        let expected = InvocationCoordinate::new(0, "test");
        assert_eq!(stack.current_path().first(), Some(&expected));
        stack.unalign();
        stack.align("test");
        // print!("{:?}", stack.current_path()); // Removed for no_std compatibility
        let expected_1 = InvocationCoordinate::new(1, "test");
        assert_eq!(stack.current_path().first(), Some(&expected_1));
        stack.unalign();
    }
}
