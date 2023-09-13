use std::fmt::{Display, Formatter, Result as FmtResult};

// TODO: sort visibility of this properly.
pub use execute::*;
pub use insert::*;
pub use normal::*;

mod execute;
mod insert;
mod normal;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mode {
    Execute,
    Insert,
    Normal(normal::Normal),
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal(normal::Normal::default())
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Execute => write!(f, "COMMAND"),
            Self::Insert => write!(f, "INSERT"),
            Self::Normal(_) => write!(f, "NORMAL"),
        }
    }
}
