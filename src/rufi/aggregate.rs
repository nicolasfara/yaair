use std::any::Any;
use std::collections::HashMap;
use std::hash::Hash;

use crate::rufi::field::Field;
use crate::rufi::messages::{InboundMessage, Path};

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

    fn branch<'a, V, Th, El>(&mut self, condition: bool, th: Th, el: El) -> V
    where
        Self: 'a,
        Th: FnOnce(&mut Self) -> V,
        El: FnOnce(&mut Self) -> V;
}

pub struct RufiEngine<D: Ord + Hash + Copy, Env, Out> {
    pub local_id: D,
    state: HashMap<Path, Box<dyn Any>>,
    inbound: InboundMessage<D>,
    outbound: OutboundMessage<D>,
    alignment_stack: AlignmentStack,
    program: fn(Env, Self) -> Out,
}
impl<Id: Ord + Hash + Copy, Env, Out> RufiEngine<Id, Env, Out> {
    pub fn new(local_id: Id, inbound: InboundMessage<Id>, program: fn(Env, Self) -> Out) -> Self {
        Self {
            local_id,
            state: HashMap::new(),
            inbound,
            outbound: OutboundMessage::empty(local_id),
            alignment_stack: AlignmentStack::new(),
            program,
        }
    }

    // fn aligned_devices(&self) -> HashSet<Id> {
    //     let current_path = Path::new(self.alignment_stack.current_path());
    //     self.inbound.devices_at_path(&current_path)
    // }
}
impl<Id: Ord + Hash + Copy, Env, Out> Aggregate<Id> for RufiEngine<Id, Env, Out> {
    fn neighboring<V>(&mut self, value: &V) -> Field<Id, V>
    where
        V: 'static + Clone,
    {
        self.alignment_stack.align("neighboring".to_string());
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
        let previous_state = self
            .state
            .get(&current_path)
            .and_then(|f| f.downcast_ref::<V>())
            .unwrap_or(initial)
            .clone();

        let updated_state = evolution(&previous_state, self);
        self.state
            .insert(current_path, Box::new(updated_state.clone()));
        updated_state
    }

    fn branch<'a, V, Th, El>(&mut self, condition: bool, th: Th, el: El) -> V
    where
        Self: 'a,
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

// fn my_program<Id: Ord + Copy + Hash, A : Aggregate<Id>>(ctx: &mut A) {
//     let my_int = ctx.repeat(&10, |x, ctx| {
//         ctx.branch(
//             0.eq(&0),
//             |ctx| ctx.neighboring(x).local(),
//             |ctx| ctx.neighboring(&50).local()
//         )
//     });
// }
