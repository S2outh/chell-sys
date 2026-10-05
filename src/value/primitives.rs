use super::ChellValue;
use super::ChellValueError;

macro_rules! le_bytes_value {
    ($($type:ident),*) => {$(
        impl ChellValue for $type {
            const MAX_BYTE_SIZE: usize = size_of::<Self>();
            #[inline]
            fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
                if bytes.len() < Self::MAX_BYTE_SIZE {
                    return Err(ChellValueError::OutOfMemory);
                }
                let value = Self::from_le_bytes(bytes[..Self::MAX_BYTE_SIZE].try_into().unwrap());
                Ok((Self::MAX_BYTE_SIZE, value))
            }
            #[inline]
            fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
                if mem.len() < Self::MAX_BYTE_SIZE {
                    return Err(ChellValueError::OutOfMemory);
                }
                let bytes = self.to_le_bytes();
                mem[..Self::MAX_BYTE_SIZE].copy_from_slice(&bytes);
                Ok(Self::MAX_BYTE_SIZE)
            }
        }
    )*}
}

// Values always encoded as little endian
le_bytes_value!(u8, i8, f32, f64);

// Little endian integers
// #[cfg(not(feature = "varint"))]
le_bytes_value!(u16, u32, u64, u128, i16, i32, i64, i128);

// Varint integers
// #[cfg(feature = "varint")]
// varint_value!(u16, u32, u64, u128, i16, i32, i64, i128);
