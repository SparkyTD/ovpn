use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct Response {
    success: bool,
    message: String,
}

impl Response {
    pub fn success(message: String) -> Response {
        return Self { success: true, message };
    }

    pub fn fail(message: String) -> Response {
        return Self { success: false, message };
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}:{}", match self.success {
            true => "ok",
            false => "err"
        }, self.message);
    }
}