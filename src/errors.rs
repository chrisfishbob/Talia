use std::{error, fmt};

// TODO: Migrate to anyhow
#[derive(Debug, Clone)]
pub struct BoardError {
    message: String,
}

impl BoardError {
    pub fn new(message: &str) -> BoardError {
        BoardError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for BoardError {}
