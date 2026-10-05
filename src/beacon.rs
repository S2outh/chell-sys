use crate::ChellDefinition;
pub mod bitfield;

// Beacon error types
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
