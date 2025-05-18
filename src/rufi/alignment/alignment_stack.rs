use std::{
    collections::{HashMap, VecDeque},
    u16,
};

use crate::rufi::messages::Path;

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
impl ToString for InvocationCoordinate {
    fn to_string(&self) -> String {
        format!("{}:{}", self.counter, self.token)
    }
}

pub(crate) struct AlignmentStack {
    stack: VecDeque<InvocationCoordinate>,
    trace: HashMap<Path, u16>,
}
impl AlignmentStack {
    pub(crate) fn new() -> Self {
        Self {
            stack: VecDeque::new(),
            trace: HashMap::new(),
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

    pub(crate) fn align_on<F, R>(&mut self, token: String, body: F) -> R
    where
        F: FnOnce() -> R,
    {
        let current_path = Path::new(self.stack.iter().cloned().collect());
        let current_counter = self
            .trace
            .get(&current_path)
            .map(|counter| counter + 1)
            .unwrap_or(0);
        let invocation_coordinate = InvocationCoordinate::new(current_counter, token);
        self.stack.push_back(invocation_coordinate);
        let result = body();
        self.stack.pop_back();
        self.trace.insert(current_path, current_counter);
        result
    }
}
