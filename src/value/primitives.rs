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

macro_rules! unsigned_varint_value {
    ($($type:ident),*) => {$(
        impl ChellValue for $type {
            const MAX_BYTE_SIZE: usize = ($type::BITS as usize).div_ceil(BITS_PER_VI_BYTE);

            #[inline]
            fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
                let mut pos = 0;
                let mut v = 0;
                loop {
                    if bytes.len() <= pos {
                        return Err(ChellValueError::OutOfMemory)
                    }

                    // extracting vi byte and continuation flag
                    let byte = bytes[pos] & VI_BYTE_MASK;
                    let flag = (bytes[pos] >> BITS_PER_VI_BYTE) != 0;
                    pos += 1;

                    // updating v
                    v = (v << BITS_PER_VI_BYTE) | byte as $type;

                    if !flag {
                        return Ok((pos, v))
                    }
                }
            }

            #[inline]
            fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
                let mut v = *self;
                let mut pos = 0;
                loop {
                    if mem.len() <= pos {
                        return Err(ChellValueError::OutOfMemory)
                    }

                    // extract byte as 7 least significant bits from remaining value
                    let byte = v as u8 & VI_BYTE_MASK;

                    // updating v
                    v = v >> BITS_PER_VI_BYTE;

                    // inserting at the beginning to keep endianness
                    mem.copy_within(0..pos, 1);
                    let flag = (pos != 0) as u8;
                    mem[0] = byte | (flag << BITS_PER_VI_BYTE);
                    pos += 1;

                    if v == 0 {
                        return Ok(pos);
                    }
                }
            }
        }
    )*};
}

macro_rules! signed_varint_value {
    ($(($type:ident, $utype:ident)),*) => {$(
        impl ZigZagValue<$type, $utype> for $type {
            #[inline]
            fn zigzag_encode(v: $type) -> $utype {
                ((v >> ($type::BITS - 1)) ^ (v << 1)) as $utype
            }
            #[inline]
            fn zigzag_decode(v: $utype) -> $type {
                (v >> 1) as $type ^ (-((v & 1) as $type))
            }
        }
        impl ChellValue for $type {
            const MAX_BYTE_SIZE: usize = $utype::MAX_BYTE_SIZE;
            #[inline]
            fn read(bytes: &[u8]) -> Result<(usize, Self), ChellValueError> {
                let (len, zv) = $utype::read(bytes)?;
                Ok((len, $type::zigzag_decode(zv)))
            }
            #[inline]
            fn write(&self, mem: &mut [u8]) -> Result<usize, ChellValueError> {
                let zv = $type::zigzag_encode(*self);
                zv.write(mem)
            }
        }
    )*};
}

// Values always encoded as plain little endian
le_bytes_value!(u8, i8, f32, f64);

// Varint integers
trait ZigZagValue<T, U> {
    fn zigzag_encode(v: T) -> U;
    fn zigzag_decode(v: U) -> T;
}
const BITS_PER_VI_BYTE: usize = 7;
const VI_BYTE_MASK: u8 = (1 << BITS_PER_VI_BYTE) - 1;

unsigned_varint_value!(u16, u32, u64, u128);
signed_varint_value!((i16, u16), (i32, u32), (i64, u64), (i128, u128));
