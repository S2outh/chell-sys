#[derive(Debug)]
pub enum ChellValueError {
    OutOfMemory,
    BadEnumVariant,
}

// # Trait definitions
pub trait ChellValue {
    const MAX_BYTE_SIZE: usize;
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError>
    where
        Self: Sized;
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError>;
}

#[cfg(feature = "ground")]
pub mod ground {
    use crate::{ChellDefinition, ChellValueError};
    use serde::ser::SerializeStruct;
    pub trait SerializableChellValue<DEF>: super::ChellValue + serde::Serialize
    where
        DEF: ChellDefinition,
    {
        fn serialize_ground(
            self,
            _def: &DEF,
            timestamp: &dyn erased_serde::Serialize,
            serializer: &dyn Fn(
                &dyn erased_serde::Serialize,
            ) -> Result<alloc::vec::Vec<u8>, erased_serde::Error>,
        ) -> Result<alloc::vec::Vec<(&'static str, alloc::vec::Vec<u8>)>, erased_serde::Error>;
    }
    pub struct GroundTelemetry<'a> {
        timestamp: &'a dyn erased_serde::Serialize,
        value: &'a dyn erased_serde::Serialize,
    }
    impl<'a> GroundTelemetry<'a> {
        pub fn new(
            timestamp: &'a dyn erased_serde::Serialize,
            value: &'a dyn erased_serde::Serialize,
        ) -> Self {
            Self { timestamp, value }
        }
    }
    impl<'a> serde::Serialize for GroundTelemetry<'a> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut s = serializer.serialize_struct("GroundTelemetry", 2)?;
            s.serialize_field("timestamp", self.timestamp)?;
            s.serialize_field("value", self.value)?;
            s.end()
        }
    }
    #[derive(Debug)]
    pub enum ReserializeError {
        ChellValueError(ChellValueError),
        SerdeError(erased_serde::Error),
    }
}

// # Primitives
macro_rules! primitive_value {
    ($type:ident) => {
        impl ChellValue for $type {
            const MAX_BYTE_SIZE: usize = size_of::<Self>();
            fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
                if bytes.len() < Self::MAX_BYTE_SIZE {
                    return Err(ChellValueError::OutOfMemory);
                }
                let value = Self::from_le_bytes(bytes[..Self::MAX_BYTE_SIZE].try_into().unwrap());
                Ok((Self::MAX_BYTE_SIZE, value))
            }
            fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
                if mem.len() < Self::MAX_BYTE_SIZE {
                    return Err(ChellValueError::OutOfMemory);
                }
                let bytes = self.to_le_bytes();
                mem[..Self::MAX_BYTE_SIZE].copy_from_slice(&bytes);
                Ok(Self::MAX_BYTE_SIZE)
            }
        }
    };
}

primitive_value!(u8);
primitive_value!(u16);
primitive_value!(u32);
primitive_value!(u64);
primitive_value!(u128);
primitive_value!(usize);

primitive_value!(i8);
primitive_value!(i16);
primitive_value!(i32);
primitive_value!(i64);
primitive_value!(i128);
primitive_value!(isize);

primitive_value!(f32);
primitive_value!(f64);

// # Empty type
impl ChellValue for () {
    const MAX_BYTE_SIZE: usize = 0;
    fn read(_bytes: &[u8]) -> Result<(usize, Self), ChellValueError>
    where
        Self: Sized,
    {
        Ok((0, ()))
    }
    fn write(&self, _mem: &mut [u8]) -> Result<usize, ChellValueError> {
        Ok(0)
    }
}

// # Arrays
impl<const N: usize, T: ChellValue> ChellValue for [T; N] {
    const MAX_BYTE_SIZE: usize = N * T::MAX_BYTE_SIZE;
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
        unsafe {
            let mut pos = 0;
            let mut arr: Self = core::mem::zeroed();
            for i in 0..N {
                if pos >= bytes.len() {
                    return Err(ChellValueError::OutOfMemory);
                }
                let (len, value) = T::read(&bytes[pos..])?;
                pos += len;
                arr[i] = value;
            }
            Ok((pos, arr))
        }
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
        let mut pos = 0;
        for i in 0..N {
            if pos >= mem.len() {
                return Err(ChellValueError::OutOfMemory);
            }
            pos += self[i].write(&mut mem[pos..])?;
        }
        Ok(pos)
    }
}
// # Vectors
// use heapless::Vec;
// impl<const N: usize, T: ChellValue> ChellValue for Vec<T, N> {
//     const MAX_BYTE_SIZE: usize = N * T::MAX_BYTE_SIZE;
//     fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
//         let (mut pos, len) = u8::read(bytes)?;
//         let mut vec = Vec::new();
//         for _ in 0..len {
//             let (len, value) = T::read(&bytes[pos..])?;
//             vec.push(value);
//             pos += len;
//         }
//         Ok((pos, vec))
//     }
//     fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
//         let mut pos = (self.len() as u8).write(mem)?;
//         for i in 0..self.len() {
//             pos += self[i].write(&mut mem[pos..])?;
//         }
//         Ok(pos)
//     }
// }

// # Options
impl<T: ChellValue> ChellValue for Option<T> {
    const MAX_BYTE_SIZE: usize = 1 + T::MAX_BYTE_SIZE;
    fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
        let mut pos = 1;
        let Some(enum_byte) = bytes.get(0) else {
            return Err(ChellValueError::OutOfMemory);
        };
        match enum_byte {
            0u8 => Ok((pos, None)),
            1u8 => {
                let (len, value) = T::read(&bytes[pos..])?;
                pos += len;
                Ok((pos, Some(value)))
            }
            _ => Err(ChellValueError::BadEnumVariant),
        }
    }
    fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
        let mut pos = 1;
        if mem.len() < 1 {
            return Err(ChellValueError::OutOfMemory);
        }
        match self {
            None => {
                mem[0] = 0u8;
            }
            Some(v0) => {
                mem[0] = 1u8;
                pos += v0.write(&mut mem[pos..])?;
            }
        }
        Ok(pos)
    }
}
