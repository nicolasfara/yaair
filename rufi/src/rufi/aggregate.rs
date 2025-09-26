use crate::rufi::alignment::alignment_stack::AlignmentStack;
use crate::rufi::data::field::Field;
use crate::rufi::data::state::State;
use crate::rufi::messages::inbound::InboundMessage;
use crate::rufi::messages::outbound::OutboundMessage;
use crate::rufi::messages::path::Path;
use crate::rufi::messages::serializer::Serializer;
use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;
use core::hash::Hash;
use serde::{Deserialize, Serialize};

pub trait Aggregate<Id: Ord + Hash + Copy + Serialize> {
    fn neighboring<V>(&mut self, value: V) -> Field<Id, V>
    where
        V: Serialize + for<'de> Deserialize<'de> + Clone + 'static;

    fn repeat<V, F>(&mut self, initial: V, evolution: F) -> V
    where
        V: Clone + 'static,
        F: FnOnce(V, &mut Self) -> V;

    fn branch<V, Th, El>(&mut self, condition: bool, th: Th, el: El) -> V
    where
        Th: FnOnce(&mut Self) -> V,
        El: FnOnce(&mut Self) -> V;
}

pub struct VM<Id: Ord + Hash + Copy + Serialize, S: Serializer> {
    pub local_id: Id,
    state: State,
    inbound: InboundMessage<Id>,
    outbound: OutboundMessage<Id>,
    alignment_stack: AlignmentStack,
    serializer: S,
}
impl<Id: Ord + Hash + Copy + Serialize, S: Serializer> VM<Id, S> {
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

    pub fn get_outbound(&self) -> Vec<u8> {
        match self.serializer.serialize(&self.outbound) {
            Ok(serialized) => serialized,
            Err(err) => panic!("Failed to serialize outbound message: {}", err),
        }
    }

    pub fn set_inbound(&mut self, inbound: InboundMessage<Id>) {
        self.inbound = inbound;
    }
}
impl<Id: Ord + Hash + Copy + Serialize, S: Serializer> Aggregate<Id> for VM<Id, S> {
    fn neighboring<V>(&mut self, value: V) -> Field<Id, V>
    where
        V: Serialize + for<'de> Deserialize<'de> + Clone + 'static,
    {
        self.alignment_stack.align("neighboring");
        let path = Path::new(self.alignment_stack.current_path());
        let result: Field<Id, V> = Field::new(
            value.clone(),
            self.inbound
                .get_at_path(&path)
                .into_iter()
                .map(|(id, elem)| match self.serializer.deserialize::<V>(&elem) {
                    Ok(deserialized_value) => (id, deserialized_value),
                    Err(err) => panic!(
                        "Failed to deserialize neighboring value from device at path {:?}: {}",
                        path, err
                    ),
                })
                .collect(),
        );
        let serialized_value = match self.serializer.serialize(&value) {
            Ok(val) => val,
            Err(err) => panic!("Failed to serialize neighboring value: {}", err),
        };
        self.outbound.append(path, serialized_value);
        self.alignment_stack.unalign();
        result
    }

    fn repeat<V, F>(&mut self, initial: V, evolution: F) -> V
    where
        V: Clone + 'static,
        F: FnOnce(V, &mut Self) -> V,
    {
        self.alignment_stack.align("repeat");
        let current_path = Path::new(self.alignment_stack.current_path());
        let previous_state = match self.state.get::<V>(&current_path) {
            Some(value) => value.clone(),
            None => initial.clone(),
        };
        let updated_state = evolution(previous_state, self);
        self.state
            .insert(current_path, Box::new(updated_state.clone()));
        self.alignment_stack.unalign();
        updated_state
    }

    fn branch<V, Th, El>(&mut self, condition: bool, th: Th, el: El) -> V
    where
        Th: FnOnce(&mut Self) -> V,
        El: FnOnce(&mut Self) -> V,
    {
        self.alignment_stack.align(format!("branch/{}", condition));
        let result = if condition { th(self) } else { el(self) };
        self.alignment_stack.unalign();
        result
    }
}
