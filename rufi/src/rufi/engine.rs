use crate::rufi::aggregate::VM;
use crate::rufi::messages::serializer::Serializer;
use crate::rufi::network::Network;
use core::hash::Hash;
use serde::Serialize;

pub struct Engine<Id, Out, Env, S, Net>
where
    Id: Ord + Hash + Copy + Serialize + for<'de> serde::Deserialize<'de>,
    S: Serializer,
    Net: Network<Id, S>,
{
    local_id: Id,
    network: Net,
    program: fn(&Env, &mut VM<Id, S>) -> Out,
    vm: VM<Id, S>,
    environment: Env,
}
impl<Id, Out, Env, S, Net> Engine<Id, Out, Env, S, Net>
where
    Id: Ord + Hash + Copy + Serialize + for<'de> serde::Deserialize<'de>,
    S: Serializer,
    Net: Network<Id, S>,
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

    const fn get_local_id(&self) -> Id {
        self.local_id
    }

    fn cycle(&mut self) -> Out {
        let inbound = self.network.prepare_inbound();
        self.vm.set_inbound(inbound);
        let result = (self.program)(&self.environment, &mut self.vm);
        let serialized_outbound = self.vm.get_outbound();
        self.network.prepare_outbound(serialized_outbound);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rufi::messages::inbound::InboundMessage;
    use alloc::vec::Vec;
    use core::fmt::{self, Display};

    // Dummy Serializer
    #[derive(Clone, Copy)]
    struct DummySerializer;
    #[derive(Debug)]
    struct DummyError;
    impl Display for DummyError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "DummyError")
        }
    }
    impl Serializer for DummySerializer {
        type Error = DummyError;
        fn serialize<T: serde::Serialize>(&self, _value: &T) -> Result<Vec<u8>, Self::Error> {
            Ok(Vec::new())
        }
        fn deserialize<T: for<'de> serde::Deserialize<'de>>(
            &self,
            _value: &[u8],
        ) -> Result<T, Self::Error> {
            Err(DummyError)
        }
    }

    // Dummy Network
    struct DummyNetwork;
    impl<Id, S> Network<Id, S> for DummyNetwork
    where
        Id: Ord + Hash + Copy + Serialize + for<'de> serde::Deserialize<'de>,
        S: Serializer,
    {
        fn prepare_outbound(&mut self, _outbound_message: Vec<u8>) {}

        fn prepare_inbound(&mut self) -> InboundMessage<Id> {
            InboundMessage::default()
        }
    }

    #[test]
    fn test_new_and_get_local_id() {
        let engine = Engine::new(1u32, DummyNetwork, (), DummySerializer, |_env, _vm| 42u8);
        assert_eq!(engine.get_local_id(), 1u32);
    }

    #[test]
    fn test_cycle() {
        let mut engine = Engine::new(2u32, DummyNetwork, (), DummySerializer, |_env, _vm| 99u8);
        let result = engine.cycle();
        assert_eq!(result, 99u8);
    }
}
