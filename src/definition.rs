use can_id::CanID;
use core::any::Any;

pub mod can_id;

// Definition error types
#[derive(Debug)]
pub struct NotFoundError;

pub trait ChellDefinition: Any {
    fn id(&self) -> CanID;
    fn address(&self) -> &str;
    fn as_any(&self) -> &dyn Any;
    #[cfg(feature = "ground")]
    fn reserialize(
        &self,
        bytes: &[u8],
        timestamp: &dyn erased_serde::Serialize,
        serializer: &dyn Fn(
            &dyn erased_serde::Serialize,
        ) -> Result<alloc::vec::Vec<u8>, erased_serde::Error>,
    ) -> Result<alloc::vec::Vec<(&'static str, alloc::vec::Vec<u8>)>, crate::ground::ReserializeError>;
}
