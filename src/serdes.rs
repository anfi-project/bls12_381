use std::convert::TryInto;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Visitor};

use crate::{Scalar, G1Affine, G1Compressed};
const SCALAR_SIZE: usize = 32;
const G1_SIZE: usize = 48;


#[derive(Clone, Debug, PartialEq)]
/// Possible errors for our serde implementation, except I can't figure out how to use anything except Message
pub enum SerdeError {
    /// Something with a message
    Message(String),
    /// An error parsing the incoming bytes
    ParsingError,
    /// Either not enough or too much data
    DataLengthError,
    /// The heck if I know
    Other,
}

// pub type SerdeResult<T> = std::result::Result<T, D: serde::de::Error>;

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerdeError::Other => f.write_str("unknown error"),
            SerdeError::ParsingError => f.write_str("parsing error"),
            SerdeError::DataLengthError => f.write_str("data length error"),
            SerdeError::Message(x) => f.write_str(x),
        }
    }
}

impl serde::de::Error for SerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl serde::ser::Error for SerdeError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self::Message(msg.to_string())
    }
}

impl std::error::Error for SerdeError {}

impl Serialize for G1Affine {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let compressed = self.to_compressed();
        return serializer.serialize_bytes(&compressed);
    }
}

impl<'de> Deserialize<'de> for G1Affine {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match deserializer.deserialize_bytes(G1AffineVisitor{}) {
            Ok(c) => { return Ok(c) }
            Err(e) => { return Err(D::Error::custom(format!("couldn't deserialize G1Affine bytes: {}", e))) }
        }
    }
}

// impl Serialize for G1Compressed {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
//         return serializer.serialize_bytes(&self.as_ref());
//     }
// }

// impl<'de> Deserialize<'de> for G1Compressed {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
//         match deserializer.deserialize_bytes(G1CompressedVisitor{}) {
//             Ok(c) => { return Ok(c) }
//             Err(e) => { return Err(D::Error::custom(format!("couldn't deserialize G1Affine bytes: {}", e))) }
//         }
//     }
// }

// #[derive(Debug)]
// /// A Visitor that knows how to read data for a G1 element, and leaves it in Compressed representation
// pub struct G1CompressedVisitor{}

// impl<'de> Visitor<'de> for G1CompressedVisitor {
//     type Value = G1Compressed;
//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("G1Affine")
//     }

//     fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> where E: Error {
//         if value.len() != G1_SIZE {
//             return Err(E::custom("incorrect data length"));
//         }
//         let mut buf = [0u8; G1_SIZE];
//         for i in 0..G1_SIZE {
//             buf[i] = value[i];
//         }
//         G1Compressed(buf);
//         let g1 = G1Affine::from_compressed(&buf);
//         if ! bool::from(g1.is_some()) {
//             return Err(E::custom("couldn't parse G1Affine bytes"));
//         }
//         return Ok(g1.unwrap())
//     }
// }


#[derive(Debug)]
/// A Visitor that knows how to read data for a G1 element, and convert it into Affine representation
pub struct G1AffineVisitor{}

impl<'de> Visitor<'de> for G1AffineVisitor {
    type Value = G1Affine;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("G1Affine")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> where E: Error {
        if value.len() != G1_SIZE {
            return Err(E::custom("incorrect data length"));
        }
        let mut buf = [0u8; G1_SIZE];
        for i in 0..G1_SIZE {
            buf[i] = value[i];
        }
        let g1 = G1Affine::from_compressed(&buf);
        if ! bool::from(g1.is_some()) {
            return Err(E::custom("couldn't parse G1Affine bytes"));
        }
        return Ok(g1.unwrap())
    }
}

impl Serialize for Scalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        // TODO switch to serializing the internal representation?
        return serializer.serialize_bytes(&self.to_bytes());
    }
}

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        match deserializer.deserialize_bytes(ScalarVisitor{}) {
            Ok(s) => { return Ok(s) }
            Err(e) => { return Err(D::Error::custom(format!("couldn't deserialize Scalar bytes: {}", e))) }
        }
    }
}

#[derive(Debug)]
/// A Visitor that knows how to read data for a Scalar
pub struct ScalarVisitor{}

impl<'de> Visitor<'de> for ScalarVisitor {
    type Value = Scalar;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("G1Affine")
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> where E: Error {
        if value.len() != SCALAR_SIZE {
            return Err(E::custom("incorrect data length"));
        }
        let mut buf = [0u8; SCALAR_SIZE];
        for i in 0..SCALAR_SIZE {
            buf[i] = value[i];
        }
        let scalar = Scalar::from_bytes(&buf);
        if ! bool::from(scalar.is_some()) {
            return Err(E::custom("couldn't parse Scalar bytes"));
        }
        return Ok(scalar.unwrap())
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E> where E: Error {
        let scalar = Scalar::from_bytes(v[0..SCALAR_SIZE].try_into().expect("OHNOES"));
        if ! bool::from(scalar.is_some()) {
            return Err(E::custom("couldn't parse Scalar bytes"));
        }
        return Ok(scalar.unwrap())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> where E: Error {
        let scalar = Scalar::from_bytes(v[0..SCALAR_SIZE].try_into().expect("OHNOES"));
        if ! bool::from(scalar.is_some()) {
            return Err(E::custom("couldn't parse Scalar bytes"));
        }
        return Ok(scalar.unwrap())
    }
}