use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct IronConfigurationError;

impl fmt::Display for IronConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Iron misconfigured!")
    }
}

impl Error for IronConfigurationError {
    fn description(&self) -> &str {
        "Iron misconfigured"
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResourceNotFound;

impl fmt::Display for ResourceNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Requested resource not found")
    }
}

impl Error for ResourceNotFound {
    fn description(&self) -> &str {
        "Requested resource not found"
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LockAcquisitionError;

impl fmt::Display for LockAcquisitionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Failed to aquire lock.")
    }
}

impl Error for LockAcquisitionError {
    fn description(&self) -> &str { "Failed to aquire lock." }
}