use crate::get_timestamp;
use std::{
    collections::VecDeque,
    sync::{LazyLock, Mutex},
};

#[derive(Debug)]
pub struct DisplayError {
    message: String,
    timestamp: u64,
}

impl DisplayError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            timestamp: get_timestamp(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

#[derive(Default)]
pub struct ErrorStack {
    pub errors: VecDeque<DisplayError>,
}

impl ErrorStack {
    pub fn push(&mut self, message: impl Into<String>) {
        self.errors.push_back(DisplayError::new(message))
    }
}

pub static ERROR_STACK: LazyLock<Mutex<ErrorStack>> =
    LazyLock::new(|| Mutex::new(ErrorStack::default()));

pub fn append_global_error(error: impl std::fmt::Display) {
    if let Ok(mut stack) = ERROR_STACK.try_lock() {
        stack.push(error.to_string());
    }
}
