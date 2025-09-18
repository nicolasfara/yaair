use crate::rufi::field::Field;
use crate::rufi::messages::{InboundMessage, Path};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use core::any::Any;
use core::hash::Hash;

use super::alignment::alignment_stack::AlignmentStack;
use super::messages::{Exportable, OutboundMessage};

pub trait Aggregate<Id: Ord + Hash + Copy> {
    fn neighboring<V>(&mut self, value: &V) -> Field<Id, V>
    where
        V: 'static + Clone;

    fn repeat<V, F>(&mut self, initial: &V, evolution: F) -> V
    where
        V: 'static + Clone,
        F: FnOnce(&V, &mut Self) -> V;

    fn branch<V, Th, El>(&mut self, condition: bool, th: Th, el: El) -> V
    where
        Th: FnOnce(&mut Self) -> V,
        El: FnOnce(&mut Self) -> V;
}

pub struct RufiEngine<D: Ord + Hash + Copy, Env, Out> {
    pub local_id: D,
    state: BTreeMap<Path, Box<dyn Any>>,
    inbound: InboundMessage<D>,
    outbound: OutboundMessage<D>,
    alignment_stack: AlignmentStack,
    program: fn(Env, &mut Self) -> Out,
}
impl<Id: Ord + Hash + Copy, Env, Out> RufiEngine<Id, Env, Out> {
    pub fn new(local_id: Id, inbound: InboundMessage<Id>, program: fn(Env, &mut Self) -> Out) -> Self {
        Self {
            local_id,
            state: BTreeMap::new(),
            inbound,
            outbound: OutboundMessage::empty(local_id),
            alignment_stack: AlignmentStack::new(),
            program,
        }
    }

    pub fn run(&mut self, env: Env) -> Out {
        (self.program)(env, self)
    }

    // fn aligned_devices(&self) -> BTreeSet<Id> {
    //     let current_path = Path::new(self.alignment_stack.current_path());
    //     self.inbound.devices_at_path(&current_path)
    // }
}
impl<Id: Ord + Hash + Copy, Env, Out> Aggregate<Id> for RufiEngine<Id, Env, Out> {
    fn neighboring<V>(&mut self, value: &V) -> Field<Id, V>
    where
        V: 'static + Clone,
    {
        self.alignment_stack.align("neighboring");
        let path = Path::new(self.alignment_stack.current_path());
        let result = Field::new(
            value.clone(),
            self.inbound
                .get_at_path::<V>(&path)
                .into_iter()
                .map(|(id, value)| (id, value.clone()))
                .collect(),
        );
        self.outbound.append(path, value.clone());
        self.alignment_stack.unalign();
        result
    }

    fn repeat<V, F>(&mut self, initial: &V, evolution: F) -> V
    where
        V: 'static + Clone,
        F: FnOnce(&V, &mut Self) -> V,
    {
        let current_path = Path::new(self.alignment_stack.current_path());
        let previous_state = match self.state.get(&current_path) {
            Some(boxed_value) => {
                match boxed_value.downcast_ref::<V>() {
                    Some(value) => value.clone(),
                    None => {
                        // Type mismatch - this indicates a serious bug in the program logic
                        // The same path is being used for different types across iterations
                        panic!(
                            "Type mismatch in repeat state at path {:?}. \
                            Expected type {} but found different type in stored state. \
                            This usually indicates the same alignment path is being used \
                            for different value types across iterations.",
                            current_path,
                            core::any::type_name::<V>()
                        );
                    }
                }
            }
            None => initial.clone(),
        };

        let updated_state = evolution(&previous_state, self);
        self.state
            .insert(current_path, Box::new(updated_state.clone()));
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
impl<Id: Ord + Hash + Copy, Env, Out> Exportable<Id> for RufiEngine<Id, Env, Out> {
    fn export(&self) -> OutboundMessage<Id> {
        todo!()
    }
}

// Example usage:
// fn my_program<Id: Ord + Copy + Hash>(env: (), ctx: &mut dyn Aggregate<Id>) -> i32 {
//     ctx.repeat(&10, |x, ctx| {
//         ctx.branch(
//             0.eq(&0),
//             |ctx| ctx.neighboring(x).local(),
//             |ctx| ctx.neighboring(&50).local()
//         )
//     })
// }
//
// Usage:
// let mut engine = RufiEngine::new(device_id, inbound_msg, my_program);
// let result = engine.run(());

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeMap;
    use crate::rufi::messages::InboundMessage;

    #[test]
    fn test_repeat_maintains_state_consistency() {
        let inbound = InboundMessage::new(BTreeMap::new());
        let program = |_env: (), _engine: &mut RufiEngine<u8, (), i32>| 42;
        let mut engine = RufiEngine::new(1u8, inbound, program);
        
        // First call should use initial value
        let result1 = engine.repeat(&10i32, |prev, _| prev + 1);
        assert_eq!(result1, 11);
        
        // Second call should use stored state
        let result2 = engine.repeat(&10i32, |prev, _| prev + 1);
        assert_eq!(result2, 12); // Should be 11 + 1, not 10 + 1
    }

    #[test]
    #[should_panic(expected = "Type mismatch in repeat state")]
    fn test_repeat_panics_on_type_mismatch() {
        let inbound = InboundMessage::new(BTreeMap::new());
        let program = |_env: (), _engine: &mut RufiEngine<u8, (), i32>| 42;
        let mut engine = RufiEngine::new(1u8, inbound, program);
        
        // Store an i32 value
        let _result1 = engine.repeat(&10i32, |prev, _| prev + 1);
        
        // Try to access the same path with a different type - should panic
        let _result2 = engine.repeat(&"hello", |prev, _| prev);
    }
}
