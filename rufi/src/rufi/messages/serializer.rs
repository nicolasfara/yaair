use alloc::vec::Vec;
use core::fmt::Display;
use serde::{Deserialize, Serialize};

pub trait Serializer {
    type Error: Display;

    fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, Self::Error>;
    fn deserialize<T: for<'de> Deserialize<'de>>(&self, value: &[u8]) -> Result<T, Self::Error>;
}
