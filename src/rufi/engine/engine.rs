use crate::rufi::aggregate::VM;
use crate::rufi::messages::path::Path;
use crate::rufi::messages::serializer::Serializer;
use crate::rufi::network::network::Network;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::any::Any;
use core::hash::Hash;

pub struct Engine<Id: Ord + Hash + Copy, Out, Env, S: Serializer, Net: Network<Id, S>> {
    local_id: Id,
    network: Net,
    program: fn(&Env, &mut VM<Id, S>) -> Out,
    vm: VM<Id, S>,
    last_state: BTreeMap<Path, Box<dyn Any>>,
    environment: Env,
}
impl<Id: Ord + Hash + Copy, Out, Env, S: Serializer, Net: Network<Id, S>>
    Engine<Id, Out, Env, S, Net>
{
    fn new(
        local_id: Id,
        network: Net,
        environment: Env,
        serializer: S,
        program: fn(&Env, &mut VM<Id, S>) -> Out,
    ) -> Self {
        Self {
            local_id,
            network,
            program,
            environment,
            vm: VM::new(local_id, serializer),
            last_state: BTreeMap::new(),
        }
    }

    fn cycle(&mut self) -> Out {
        let inbound = self.network.prepare_inbound();
        self.vm.set_inbound(inbound);
        let result = (self.program)(&self.environment, &mut self.vm);
        let new_state = self.vm.state_snapshot();
        self.last_state = new_state;
        let outbound = self.vm.get_outbound();
        self.network.prepare_outbound(outbound);
        result
    }
}
