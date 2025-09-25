use crate::rufi::aggregate::VM;
use crate::rufi::messages::serializer::Serializer;
use crate::rufi::network::network::Network;
use core::hash::Hash;

pub struct Engine<Id: Ord + Hash + Copy, Out, Env, S: Serializer, Net: Network<Id, S>> {
    local_id: Id,
    network: Net,
    program: fn(&Env, &mut VM<Id, S>) -> Out,
    vm: VM<Id, S>,
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
        }
    }

    fn cycle(&mut self) -> Out {
        let inbound = self.network.prepare_inbound();
        self.vm.set_inbound(inbound);
        let result = (self.program)(&self.environment, &mut self.vm);
        let outbound = self.vm.get_outbound();
        self.network.prepare_outbound(outbound);
        result
    }
}
