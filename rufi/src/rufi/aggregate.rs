use crate::rufi::alignment::alignment_stack::AlignmentStack;
use crate::rufi::data::field::Field;
use crate::rufi::data::state::State;
use crate::rufi::messages::inbound::InboundMessage;
use crate::rufi::messages::outbound::OutboundMessage;
use crate::rufi::messages::path::Path;
use crate::rufi::messages::serializer::Serializer;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::hash::Hash;
use serde::{Deserialize, Serialize};

/// Represents errors that can occur during aggregate computation
#[derive(Debug, Eq, PartialEq)]
pub enum AggregateError {
    SerializationError(String),
    DeserializationError(String),
}

impl core::fmt::Display for AggregateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SerializationError(msg) => write!(f, "Serialization error: {msg}"),
            Self::DeserializationError(msg) => {
                write!(f, "Deserialization error: {msg}")
            }
        }
    }
}

/// Main trait for aggregate computing operations.
///
/// This trait provides the core operations for distributed aggregate computing:
/// - `neighboring`: Share values with neighboring devices
/// - `repeat`: Maintain state across computation rounds
/// - `branch`: Conditional execution with alignment
pub trait Aggregate<Id: Ord + Hash + Copy + Serialize> {
    /// Share a value with neighboring devices and collect their values.
    ///
    /// # Arguments
    /// * `value` - The value to share with neighbors
    ///
    /// # Returns
    /// A `Field` containing the local value and neighboring values
    fn neighboring<V>(&mut self, value: &V) -> Result<Field<Id, V>, AggregateError>
    where
        V: Serialize + for<'de> Deserialize<'de> + Clone + 'static;

    /// Maintain state across computation rounds with evolution function.
    ///
    /// # Arguments
    /// * `initial` - Initial value if no previous state exists
    /// * `evolution` - Function to evolve the state
    ///
    /// # Returns
    /// The evolved state value
    fn repeat<V, F>(&mut self, initial: &V, evolution: F) -> V
    where
        V: Clone + 'static,
        F: FnOnce(V, &mut Self) -> V;

    /// Conditional execution with proper alignment.
    ///
    /// # Arguments
    /// * `condition` - Boolean condition to determine branch
    /// * `th` - Function to execute if condition is true
    /// * `el` - Function to execute if condition is false
    ///
    /// # Returns
    /// Result of the executed branch
    fn branch<V, Th, El>(&mut self, condition: bool, th: Th, el: El) -> V
    where
        Th: FnOnce(&mut Self) -> V,
        El: FnOnce(&mut Self) -> V;
}

/// Virtual Machine implementation for aggregate computing.
///
/// Manages state, message passing, and alignment for distributed computation.
pub struct VM<Id: Ord + Hash + Copy + Serialize, S: Serializer> {
    pub local_id: Id,
    state: State,
    inbound: InboundMessage<Id>,
    outbound: OutboundMessage<Id>,
    alignment_stack: AlignmentStack,
    serializer: S,
}

impl<Id: Ord + Hash + Copy + Serialize, S: Serializer> VM<Id, S> {
    /// Create a new VM instance with default state.
    pub fn new(local_id: Id, serializer: S) -> Self {
        Self {
            local_id,
            state: State::default(),
            inbound: InboundMessage::default(),
            outbound: OutboundMessage::empty(local_id),
            alignment_stack: AlignmentStack::new(),
            serializer,
        }
    }

    /// Create a new VM instance with provided state.
    pub fn new_with_state(local_id: Id, serializer: S, state: State) -> Self {
        Self {
            local_id,
            state,
            inbound: InboundMessage::default(),
            outbound: OutboundMessage::empty(local_id),
            alignment_stack: AlignmentStack::new(),
            serializer,
        }
    }

    /// Get the serialized outbound message.
    ///
    /// # Returns
    /// Serialized outbound message as bytes, or panics on serialization error
    pub fn get_outbound(&self) -> Result<Vec<u8>, AggregateError> {
        self.serializer.serialize(&self.outbound).map_err(|err| {
            AggregateError::SerializationError(format!(
                "Failed to serialize outbound message: {err}",
            ))
        })
    }

    pub fn prepare_new_round(&mut self, inbound: InboundMessage<Id>) {
        self.outbound = OutboundMessage::empty(self.local_id);
        self.alignment_stack = AlignmentStack::new();
        self.inbound = inbound;
    }
}

impl<Id: Ord + Hash + Copy + Serialize, S: Serializer> Aggregate<Id> for VM<Id, S> {
    fn neighboring<V>(&mut self, value: &V) -> Result<Field<Id, V>, AggregateError>
    where
        V: Serialize + for<'de> Deserialize<'de> + Clone + 'static,
    {
        self.alignment_stack.align("neighboring");
        let path = Path::new(self.alignment_stack.current_path());

        // Collect neighboring values with improved error handling
        let mut neighboring_values = alloc::collections::BTreeMap::new();
        for (id, elem) in self.inbound.get_at_path(&path) {
            match self.serializer.deserialize::<V>(&elem) {
                Ok(deserialized_value) => {
                    neighboring_values.insert(id, deserialized_value);
                }
                Err(err) => {
                    self.alignment_stack.unalign();
                    return Err(AggregateError::DeserializationError(format!(
                        "Failed to deserialize neighboring value at path {path}: {err}",
                    )));
                }
            }
        }

        let result = Field::new(value.clone(), neighboring_values);

        // Serialize and append to outbound
        let serialized_value = self.serializer.serialize(&value).map_err(|err| {
            self.alignment_stack.unalign();
            AggregateError::SerializationError(format!(
                "Failed to serialize neighboring value: {err}"
            ))
        })?;

        self.outbound.append(&path, serialized_value);
        self.alignment_stack.unalign();
        Ok(result)
    }

    fn repeat<V, F>(&mut self, initial: &V, evolution: F) -> V
    where
        V: Clone + 'static,
        F: FnOnce(V, &mut Self) -> V,
    {
        self.alignment_stack.align("repeat");
        let current_path = Path::new(self.alignment_stack.current_path());
        let previous_state = self
            .state
            .get::<V>(&current_path)
            .map_or_else(|| initial.clone(), Clone::clone);
        let updated_state = evolution(previous_state, self);
        self.state.insert(current_path, updated_state.clone());
        self.alignment_stack.unalign();
        updated_state
    }

    fn branch<V, Th, El>(&mut self, condition: bool, th: Th, el: El) -> V
    where
        Th: FnOnce(&mut Self) -> V,
        El: FnOnce(&mut Self) -> V,
    {
        self.alignment_stack.align(format!("branch/{condition}"));
        let result = if condition { th(self) } else { el(self) };
        self.alignment_stack.unalign();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::boxed::Box;
    use alloc::collections::BTreeMap;
    use alloc::vec;
    use core::any::Any;

    // Mock serializer for testing
    struct MockSerializer;

    impl Serializer for MockSerializer {
        type Error = &'static str;

        fn serialize<T: Serialize>(&self, _value: &T) -> Result<Vec<u8>, Self::Error> {
            Ok(vec![1, 2, 3])
        }

        fn deserialize<T: for<'de> Deserialize<'de>>(
            &self,
            _value: &[u8],
        ) -> Result<T, Self::Error> {
            Err("Mock deserialization not implemented")
        }
    }

    #[test]
    fn test_vm_creation() {
        let vm = VM::new(42u32, MockSerializer);
        assert_eq!(vm.local_id, 42);
    }

    #[test]
    fn repeat_should_return_initial_on_first_call() {
        let mut vm = VM::new(1u32, MockSerializer);
        let initial_value = 10;
        let result = vm.repeat(&initial_value, |state, _| state + 1);
        assert_eq!(result, initial_value + 1);
    }

    #[test]
    fn repeat_should_use_last_available_state() {
        let mut state_map: BTreeMap<Path, Box<dyn Any>> = BTreeMap::new();
        state_map.insert(Path::from("repeat:0"), Box::new(20));
        let state = State::from_snapshot(state_map);
        let mut vm = VM::new_with_state(1, MockSerializer, state);
        let initial_value = 10;
        let result = vm.repeat(&initial_value, |prev, _| prev + 1);
        assert_eq!(result, 21); // 20 from state + 1 from evolution
        vm.prepare_new_round(InboundMessage::default());
        let next_result = vm.repeat(&initial_value, |prev, _| prev + 1);
        assert_eq!(next_result, 22); // 21 from previous + 1 from evolution
    }
}
