use crate::yaair::aggregate::{AggregateError, VM};
use crate::yaair::messages::serializer::Serializer;
use crate::yaair::network::Network;
use core::hash::Hash;
use serde::Serialize;

pub struct Engine<Id, Out, Env, S, Net>
where
    Id: Ord + Hash + Copy + Serialize + for<'de> serde::Deserialize<'de>,
    S: Serializer,
    Net: Network<Id>,
{
    network: Net,
    program: fn(&Env, &mut VM<Id, S>) -> Result<Out, AggregateError>,
    vm: VM<Id, S>,
    environment: Env,
}
impl<Id, Out, Env, S, Net> Engine<Id, Out, Env, S, Net>
where
    Id: Ord + Hash + Copy + Serialize + for<'de> serde::Deserialize<'de>,
    S: Serializer,
    Net: Network<Id>,
{
    pub fn new(
        network: Net,
        environment: Env,
        serializer: S,
        program: fn(&Env, &mut VM<Id, S>) -> Result<Out, AggregateError>,
    ) -> Self {
        let local_id = network.get_local_id();
        Self {
            network,
            program,
            environment,
            vm: VM::new(local_id, serializer),
        }
    }

    pub fn get_local_id(&self) -> Id {
        self.network.get_local_id()
    }

    pub fn cycle(&mut self) -> Result<Out, AggregateError> {
        let inbound = self.network.prepare_inbound();
        self.vm.prepare_new_round(inbound);
        let result = (self.program)(&self.environment, &mut self.vm)?;
        let serialized_outbound = self.vm.get_outbound()?;
        self.network.prepare_outbound(serialized_outbound);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaair::messages::inbound::InboundMessage;
    #[cfg(not(feature = "std"))]
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
    struct DummyNetwork<Id>(pub Id);
    impl<Id> Network<Id> for DummyNetwork<Id>
    where
        Id: Ord + Hash + Copy + Serialize + for<'de> serde::Deserialize<'de>,
    {
        fn get_local_id(&self) -> Id {
            self.0
        }

        fn prepare_outbound(&mut self, _outbound_message: Vec<u8>) {}

        fn prepare_inbound(&mut self) -> InboundMessage<Id> {
            InboundMessage::default()
        }
    }

    #[test]
    fn test_new_and_get_local_id() {
        let engine = Engine::new(DummyNetwork(1u32), (), &DummySerializer, |_env, _vm| {
            Ok(42u8)
        });
        assert_eq!(engine.get_local_id(), 1u32);
    }

    #[test]
    fn test_cycle() {
        let mut engine = Engine::new( DummyNetwork(2u32), (), &DummySerializer, |_env, _vm| {
            Ok(99u8)
        });
        let result = engine.cycle();
        assert_eq!(result, Ok(99u8));
    }

    #[test]
    fn cycle_should_return_program_error() {
        let mut engine = Engine::new( DummyNetwork(2u32), (), &DummySerializer, |_env, _vm| {
            Err(AggregateError::DeserializationError(
                "program failed".into(),
            ))
        });
        let result: Result<u8, AggregateError> = engine.cycle();
        assert_eq!(
            result,
            Err(AggregateError::DeserializationError(
                "program failed".into()
            ))
        );
    }
}
