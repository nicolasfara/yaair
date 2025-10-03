use crate::rufi::alignment::alignment_stack::AlignmentStack;
use crate::rufi::data::field::Field;
use crate::rufi::data::state::State;
use crate::rufi::messages::inbound::InboundMessage;
use crate::rufi::messages::outbound::OutboundMessage;
use crate::rufi::messages::path::Path;
use crate::rufi::messages::serializer::Serializer;
use alloc::collections::BTreeMap;
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

    fn share<V, E>(&mut self, initial: &V, evolution: E) -> Result<V, AggregateError>
    where
        V: Serialize + for<'de> Deserialize<'de> + Clone + 'static,
        E: FnOnce(&mut Self, Field<Id, V>) -> V;
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

    fn get_at_path<V>(&self, path: &Path) -> Result<BTreeMap<Id, V>, AggregateError>
    where
        V: for<'de> Deserialize<'de>,
    {
        let mut result = BTreeMap::new();
        for (id, elem) in self.inbound.get_at_path(path) {
            match self.serializer.deserialize::<V>(&elem) {
                Ok(deserialized_value) => {
                    result.insert(id, deserialized_value);
                }
                Err(err) => {
                    return Err(AggregateError::DeserializationError(format!(
                        "Failed to deserialize value at path {path}: {err}",
                    )));
                }
            }
        }
        Ok(result)
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
        let neighboring_values = self.get_at_path(&path)?;

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
        self.alignment_stack.align(format!("branch[{condition}]"));
        let result = if condition { th(self) } else { el(self) };
        self.alignment_stack.unalign();
        result
    }

    fn share<V, E>(&mut self, initial: &V, evolution: E) -> Result<V, AggregateError>
    where
        V: Serialize + for<'de> Deserialize<'de> + Clone + 'static,
        E: FnOnce(&mut Self, Field<Id, V>) -> V,
    {
        self.alignment_stack.align("share");
        let current_path = Path::new(self.alignment_stack.current_path());
        let previous_state = self
            .state
            .get::<V>(&current_path)
            .map_or_else(|| initial.clone(), Clone::clone);
        let neighboring_values = self.get_at_path(&current_path)?;
        let field = Field::new(previous_state, neighboring_values);
        let updated_state = evolution(self, field);
        self.state
            .insert(current_path.clone(), updated_state.clone());
        let serialized_value = self.serializer.serialize(&updated_state).map_err(|err| {
            self.alignment_stack.unalign();
            AggregateError::SerializationError(format!("Failed to serialize share value: {err}"))
        })?;
        self.outbound.append(&current_path, serialized_value);
        self.alignment_stack.unalign();
        Ok(updated_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rufi::messages::valuetree::ValueTree;
    use alloc::boxed::Box;
    use alloc::collections::BTreeMap;
    use core::any::Any;

    // Mock serializer for testing
    struct MockSerializer;

    impl Serializer for MockSerializer {
        type Error = serde_json::Error;

        fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
            serde_json::to_vec(value)
        }

        fn deserialize<T: for<'de> Deserialize<'de>>(
            &self,
            value: &[u8],
        ) -> Result<T, Self::Error> {
            serde_json::from_slice(value)
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

    #[test]
    fn neighboring_should_return_a_field_with_only_local_value() {
        let mut vm = VM::new(1u32, MockSerializer);
        let value = 100u32;
        let field = vm.neighboring(&value).unwrap();
        let expected_field = Field::new(value, BTreeMap::new());
        assert_eq!(field, expected_field);
    }

    #[test]
    fn neighboring_should_create_a_field_with_aligned_devices() {
        let serializer = MockSerializer;
        let path = Path::from("neighboring:0");
        let value_device_1 = serializer.serialize(&1u32).unwrap();
        let value_device_2 = serializer.serialize(&2u32).unwrap();
        let device_1 = ValueTree::new(BTreeMap::from([(path.clone(), value_device_1)]));
        let device_2 = ValueTree::new(BTreeMap::from([(path, value_device_2)]));
        let inbound_map: BTreeMap<u32, ValueTree> =
            BTreeMap::from([(1u32, device_1), (2u32, device_2)]);
        let inbound = InboundMessage::new(inbound_map);
        let mut vm = VM::new(0u32, MockSerializer);
        vm.prepare_new_round(inbound);
        let field = vm.neighboring(&1u32).unwrap();
        let expected_field = Field::new(1u32, BTreeMap::from([(1u32, 1u32), (2u32, 2u32)]));
        assert_eq!(field, expected_field);
    }

    #[test]
    fn branch_should_project_field_on_aligned_devices() {
        let serializer = MockSerializer;
        let path_even = Path::from("branch[true]:0/neighboring:0");
        let path_odd = Path::from("branch[false]:0/neighboring:0");
        let value_device_1 = serializer.serialize(&1u32).unwrap();
        let value_device_2 = serializer.serialize(&2u32).unwrap();
        let device_1 = ValueTree::new(BTreeMap::from([(path_odd, value_device_1)]));
        let device_2 = ValueTree::new(BTreeMap::from([(path_even, value_device_2)]));
        let inbound_map: BTreeMap<u32, ValueTree> =
            BTreeMap::from([(1u32, device_1), (2u32, device_2)]);
        let inbound = InboundMessage::new(inbound_map);
        let mut vm = VM::new(0u32, MockSerializer);
        vm.prepare_new_round(inbound);
        let field = vm.branch(
            vm.local_id.is_multiple_of(2),
            |vm| vm.neighboring(&u32::MAX).unwrap(),
            |vm| vm.neighboring(&u32::MIN).unwrap(),
        );
        let expected_field = Field::new(u32::MAX, BTreeMap::from([(2u32, 2u32)]));
        assert_eq!(field, expected_field);
    }

    #[test]
    fn share_should_use_initial_value_when_no_previous_state() {
        let serializer = MockSerializer;
        let mut vm = VM::new(1u32, MockSerializer);
        let initial_value = 42;
        let result = vm
            .share(&initial_value, |_, field| field.local() * 2)
            .unwrap();
        assert_eq!(result, initial_value * 2);
        let to_send = serializer
            .deserialize::<OutboundMessage<u32>>(vm.get_outbound().unwrap().as_slice())
            .unwrap();
        let sent_value = to_send.at(&Path::from("share:0")).unwrap();
        let deserialized_sent_value = serializer.deserialize::<i32>(sent_value).unwrap();
        assert_eq!(deserialized_sent_value, initial_value * 2);
    }

    #[test]
    fn share_should_use_last_state_and_neighbors() {
        fn program(vm: &mut VM<u32, MockSerializer>) -> Result<i32, AggregateError> {
            let initial_value = 1i32;
            vm.share(&initial_value, |_, field| {
                let size: i32 = field.size().try_into().unwrap();
                field.local() + size
            })
        }
        let serializer = MockSerializer;
        let path = Path::from("share:0");
        let value_device_1 = serializer.serialize(&10i32).unwrap();
        let value_device_2 = serializer.serialize(&20i32).unwrap();
        let device_1 = ValueTree::new(BTreeMap::from([(path.clone(), value_device_1)]));
        let device_2 = ValueTree::new(BTreeMap::from([(path, value_device_2)]));
        let inbound_map: BTreeMap<u32, ValueTree> =
            BTreeMap::from([(1u32, device_1), (2u32, device_2)]);
        let inbound = InboundMessage::new(inbound_map);
        let mut vm = VM::new(0u32, MockSerializer);
        vm.prepare_new_round(inbound);
        let result = program(&mut vm).unwrap();
        assert_eq!(result, 4);
        // Reset neighbors, but the state should persist
        vm.prepare_new_round(InboundMessage::default());
        let next_result = program(&mut vm).unwrap();
        assert_eq!(next_result, 5);
    }
}
