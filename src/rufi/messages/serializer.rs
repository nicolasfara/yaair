use core::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_value::Value;

pub trait Serializer {
    type Error: Display;

    fn serialize<T: Serialize>(&self, value: &T) -> Result<Value, Self::Error>;
    fn deserialize<T: for<'de> Deserialize<'de>>(&self, value: &Value) -> Result<T, Self::Error>;
}
