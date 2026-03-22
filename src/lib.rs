#![no_std]
#![feature(const_trait_impl)]

#[cfg(feature = "ground")]
extern crate alloc;

mod bitfield;
mod chell_union;
mod chell_value;

use core::any::Any;

// macro reexports
pub use macros::ChellValue;
pub use macros::beacon;
pub use macros::chell_definition;

// value reexports
pub use chell_value::ChellValue;
pub use chell_value::ChellValueError;

// container reexports
pub use chell_union::ChellUnion;
pub use chell_union::UnsupportedValue;
pub use chell_union::ceil_to_fd_compat;

#[macro_export]
macro_rules! match_value {
    ($value:expr, {
        $($t:ty => $body:expr,)*
    }) => {{
        let any = $value.as_any();
        $(if any.is::<$t>() {
            $body
        })else*
    }}
}

pub const trait ChellDefinition: Any {
    fn id(&self) -> u16;
    fn address(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
}

#[cfg(feature = "ground")]
pub use crate::chell_value::ground;
/// Reexports that should only be used by the macro generated code
pub mod _internal {
    use crate::ChellValue;
    pub use crate::bitfield::Bitfield;
    #[cfg(feature = "ground")]
    pub use crate::ground::*;
    pub const trait InternalChellDefinition: crate::ChellDefinition {
        type ChellValueType: crate::ChellValue;
        const MAX_BYTE_SIZE: usize = Self::ChellValueType::MAX_BYTE_SIZE;
        const ID: u16;
    }
}

// Error types
#[derive(Debug)]
pub struct NotFoundError;

#[derive(Debug)]
pub enum BeaconOperationError {
    DefNotInBeacon,
    OutOfMemory,
}

#[derive(Debug)]
pub enum ParseError {
    WrongId,
    BadCRC,
    OutOfMemory,
}

// Dynamic beacon trait
pub trait Beacon {
    type Timestamp;
    fn insert_slice(
        &mut self,
        chell_definition: &dyn ChellDefinition,
        bytes: &[u8],
    ) -> Result<(), BeaconOperationError>;
    fn from_bytes(
        &mut self,
        bytes: &[u8],
        crc_func: &mut dyn FnMut(&[u8]) -> u16,
    ) -> Result<(), ParseError>;
    fn to_bytes(&mut self, crc_func: &mut dyn FnMut(&[u8]) -> u16) -> &[u8];
    fn set_timestamp(&mut self, timestamp: Self::Timestamp);
    fn flush(&mut self);
    fn name(&self) -> &'static str;
    fn id(&self) -> u8;
}
