use crate::yaair::messages::inbound::InboundMessage;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::hash::Hash;
use serde::{Deserialize, Serialize};

pub trait Network<Id: Ord + Hash + Copy + Serialize + for<'de> Deserialize<'de>> {
    fn prepare_outbound(&mut self, outbound_message: Vec<u8>);
    fn prepare_inbound(&mut self) -> InboundMessage<Id>;
}
