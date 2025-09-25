use crate::rufi::messages::inbound::InboundMessage;
use crate::rufi::messages::outbound::OutboundMessage;
use crate::rufi::messages::serializer::Serializer;
use core::hash::Hash;

pub trait Network<Id: Ord + Hash + Copy, S: Serializer> {
    fn prepare_outbound(&mut self, outbound_message: OutboundMessage<Id>);
    fn prepare_inbound(&mut self) -> InboundMessage<Id>;
}
