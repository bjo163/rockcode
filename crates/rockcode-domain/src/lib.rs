#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use core::fmt;

/// Opaque prefixed IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
    pub fn new(prefix: &str, uuid: &uuid::Uuid) -> Self {
        let mut s = String::from(prefix);
        s.push_str(&uuid.to_string());
        Id(s)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum DomainError {
    InvalidId,
    InvalidStateTransition,
    NotFound,
}
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InvalidId => write!(f, "invalid id"),
            DomainError::InvalidStateTransition => write!(f, "invalid state transition"),
            DomainError::NotFound => write!(f, "not found"),
        }
    }
}
impl core::error::Error for DomainError {}
