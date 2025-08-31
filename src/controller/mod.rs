mod epd;
mod inky;

use crate::AppError;
use axum::http::StatusCode;

pub use epd::EPDType;
pub use inky::Inky;

#[derive(Debug)]
#[repr(C)]
struct PascalString {
    len: u8,
    pub chars: [u8; u8::MAX as usize],
}

impl PascalString {
    fn with_len(len: u8) -> Self {
        Self {
            len,
            chars: [0; u8::MAX as usize],
        }
    }
}

impl std::fmt::Display for PascalString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = str::from_utf8(&self.chars).unwrap();
        writeln!(f, "{x}")
    }
}

pub async fn blink() -> Result<StatusCode, AppError> {
    Ok(StatusCode::OK)
}
