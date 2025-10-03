use rufi::rufi::aggregate::{Aggregate, AggregateError, VM};
use rufi::rufi::data::field::Field;
use rufi::rufi::engine::Engine;
use rufi::rufi::messages::inbound::InboundMessage;
use rufi::rufi::network::Network;
use rufi_serde::rufi_serde::json::JsonSerializer;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;

struct GradientEnv {
    pub is_source: bool,
}
impl GradientEnv {
    fn distances(&self) -> Field<u32, f32> {
        Field::new(0.0, BTreeMap::from([(1, 1.0), (2, 2.0), (3, 1.5)]))
    }
}

struct DummyNetwork;
impl Network<u32, JsonSerializer> for DummyNetwork {
    fn prepare_outbound(&mut self, _outbound_message: Vec<u8>) {}

    fn prepare_inbound(&mut self) -> InboundMessage<u32> {
        InboundMessage::default()
    }
}

#[allow(clippy::print_stdout, clippy::print_stderr, clippy::use_debug)]
pub fn main() {
    let env = GradientEnv { is_source: false };
    let mut engine = Engine::new(0u32, DummyNetwork, env, JsonSerializer, gradient);
    for _ in 0..10 {
        match engine.cycle() {
            Ok(result) => println!("Gradient result: {result:?}"),
            Err(e) => eprintln!("Error during cycle: {e:?}"),
        }
        sleep(Duration::from_secs(1));
    }
}

fn gradient(env: &GradientEnv, vm: &mut VM<u32, JsonSerializer>) -> Result<f32, AggregateError> {
    let initial = f32::MAX;
    vm.share(&initial, |_, field| {
        let distances = field.aligned_map(&env.distances(), |a, b| a + b);
        let min_distance =
            *distances.min_by(|a, b| PartialOrd::partial_cmp(&a, &b).unwrap_or(Ordering::Greater));
        if env.is_source {
            0.0
        } else {
            min_distance
        }
    })
}
