use crate::yaair_serde::json::JsonSerializer;
use serde::{Deserialize, Serialize};
use yaair::yaair::aggregate::{Aggregate, VM};
use yaair::yaair::messages::outbound::OutboundMessage;
use yaair::yaair::messages::serializer::Serializer;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
struct Dummy {
    a: i32,
    b: String,
}

#[test]
fn test_neighboring_serialization() {
    let serializer = JsonSerializer;
    let mut vm = VM::new(1u32, JsonSerializer);
    let value = Dummy {
        a: 7,
        b: "test".to_string(),
    };
    vm.neighboring(&value)
        .expect("neighboring should serialize");
    let outbound_bytes = vm.get_outbound().expect("outbound should serialize");
    let outbound = serializer
        .deserialize::<OutboundMessage<u32>>(&outbound_bytes)
        .unwrap();
    let val = outbound.at(&"neighboring:0".into()).unwrap();
    let deserialized: Dummy = serializer.deserialize(val).unwrap();
    assert_eq!(deserialized, value);
}
