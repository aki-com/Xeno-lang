/// エラー型
#[derive(Debug)]
pub struct XenoError {
    pub message: String,
}

impl XenoError {
    pub fn new(msg: impl Into<String>) -> Self {
        XenoError {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for XenoError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

pub type Result<T> = std::result::Result<T, XenoError>;
