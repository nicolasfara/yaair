use core::hash::Hash;
use crate::rufi::aggregate::VM;
use crate::rufi::network::network::Network;

pub struct Engine<Id: Ord + Hash + Copy, Out, Env, Net: Network<Id>> {
    local_id: Id,
    network: Net,
    program: fn(Env, &mut VM<Id>) -> Out,
    vm: VM<Id>,
}
impl <Id: Ord + Hash + Copy, Out, Env, Net: Network<Id>> Engine<Id, Out, Env, Net> {
    fn new(local_id: Id, network: Net, program: fn(Env, &mut VM<Id>) -> Out) -> Self {
        Self {
            local_id,
            network,
            program,
            vm: VM::new(local_id)
        }
    }

    fn cycle(&mut self) -> Out {
        todo!()
    }
}
