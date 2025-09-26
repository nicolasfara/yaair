#[cfg(test)]
mod aggregate_json_test {
    use rufi::rufi::aggregate::{Aggregate, VM};
    use rufi::rufi::messages::outbound::OutboundMessage;
    use rufi::rufi::messages::serializer::Serializer;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    struct Dummy {
        a: i32,
        b: String,
    }

    struct JsonSerializer;
    impl Serializer for JsonSerializer {
        type Error = serde_json::Error;
        fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
            serde_json::to_vec(value)
        }
        fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8]) -> Result<T, Self::Error> {
            serde_json::from_slice(data)
        }
    }

    #[test]
    fn test_neighboring_serialization() {
        let serializer = JsonSerializer;
        let mut vm = VM::new(1u32, JsonSerializer);
        let value = Dummy {
            a: 7,
            b: "test".to_string(),
        };
        let _ = vm.neighboring(value.clone());
        let outbound_bytes = vm.get_outbound();
        // Deserialize outbound as OutboundMessage
        let outbound = serializer
            .deserialize::<OutboundMessage<u32>>(&outbound_bytes)
            .unwrap();
        let val = outbound.at(&"neighboring:0".into()).unwrap();
        let deserialized: Dummy = serializer.deserialize(val).unwrap();
        assert_eq!(deserialized, value.clone());
    }
}
