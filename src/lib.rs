#![no_std]
#![feature(const_trait_impl)]
#![feature(array_try_from_fn)]

#[cfg(feature = "ground")]
extern crate alloc;

pub mod beacon;
pub mod definition;
mod proc_macros;
pub mod union;
pub mod value;

/// Reexports of macros
pub use macros::ChellValue;
pub use macros::beacon;
pub use macros::chell_definition;

/// Reexports of most relevant traits
pub use beacon::Beacon;

pub use definition::ChellDefinition;
pub use definition::can_id::CanID;

pub use value::ChellValue;
pub use value::ChellValueError;
pub use value::ParsableChellValue;

pub use union::ChellUnion;

/// Reexports for ground
#[cfg(feature = "ground")]
pub use crate::value::ground;

/// Reexports that should only be used by the macro generated code
pub mod _internal {
    use crate::ChellValue;
    pub use crate::beacon::bitfield::Bitfield;
    #[cfg(feature = "ground")]
    pub use crate::ground::*;
    pub const trait InternalChellDefinition: crate::ChellDefinition {
        type ChellValueType: crate::ChellValue;
        const MAX_BYTE_SIZE: usize = Self::ChellValueType::MAX_BYTE_SIZE;
    }
}
