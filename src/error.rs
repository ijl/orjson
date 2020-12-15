use std::fmt;

#[derive(Debug, Clone)]
pub struct DeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub jpart: String,
}

impl fmt::Display for DeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
