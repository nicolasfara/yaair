use rufi::rufi::messages::serializer::Serializer;
use serde::{Deserialize, Serialize};

#[cfg(feature = "json")]
pub struct JsonSerializer;
impl Serializer for JsonSerializer {
    type Error = serde_json::Error;

    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(value)
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, value: &[u8]) -> Result<T, Self::Error> {
        serde_json::from_slice(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Dummy {
        a: i32,
        b: String,
    }

    #[test]
    fn test_serialize_deserialize_struct() {
        let serializer = JsonSerializer;
        let value = Dummy {
            a: 42,
            b: "ciao".to_string(),
        };
        let bytes = serializer.serialize(&value).expect("serialize ok");
        let result: Dummy = serializer.deserialize(&bytes).expect("deserialize ok");
        assert_eq!(value, result);
    }

    #[test]
    fn test_serialize_deserialize_int() {
        let serializer = JsonSerializer;
        let value = 123i32;
        let bytes = serializer.serialize(&value).expect("serialize ok");
        let result: i32 = serializer.deserialize(&bytes).expect("deserialize ok");
        assert_eq!(value, result);
    }
}
