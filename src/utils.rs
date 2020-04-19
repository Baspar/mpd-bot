pub type BoxError = std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>;

#[derive(Debug)]
pub struct CustomError {
    reason: String
}

impl CustomError {
    pub fn new(reason: String) -> CustomError {
        CustomError { reason }
    }
}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for CustomError { }
