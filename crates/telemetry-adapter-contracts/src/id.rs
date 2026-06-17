/// AdapterId type for identifying adapters.
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::error::{AdapterError, AdapterResult};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AdapterId(String);

impl AdapterId {
    pub fn new(id: impl Into<String>) -> AdapterResult<Self> {
        let s = id.into();
        if s.is_empty() {
            return Err(AdapterError::InitializationFailed(
                "AdapterId must not be empty".into(),
            ));
        }
        Ok(AdapterId(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for AdapterId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AdapterId {
    type Err = AdapterError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AdapterId::new(s.to_owned())
    }
}
