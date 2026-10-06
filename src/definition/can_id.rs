// CanID Error types
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("Can id out of range")]
pub struct IdOutOfRange;

#[derive(Clone, Copy)]
pub enum CanID {
    Single(u16),
    Range(u16, u16),
}

impl CanID {
    pub fn get(&self, offset: u16) -> Result<u16, IdOutOfRange> {
        match *self {
            Self::Single(v) => Ok(v),
            Self::Range(b, len) => {
                if offset < len {
                    Ok(b + offset)
                } else {
                    Err(IdOutOfRange)
                }
            }
        }
    }
    pub fn offset(&self, id: u16) -> Result<u16, IdOutOfRange> {
        match *self {
            Self::Single(v) => {
                if id == v {
                    Ok(0)
                } else {
                    Err(IdOutOfRange)
                }
            }
            Self::Range(b, len) => {
                if id >= b && id < b + len {
                    Ok(id - b)
                } else {
                    Err(IdOutOfRange)
                }
            }
        }
    }
}
