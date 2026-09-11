use serde::Serialize;

/// Every command returns this on failure. It serializes to `{ code, message }` so the
/// frontend can branch on `code` without string-matching prose.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("{0}")]
    Invalid(String),

    /// Raised by the real providers in Phase 4; the stub resolver cannot fail this way yet.
    #[allow(dead_code)]
    #[error("metadata could not be resolved: {0}")]
    Resolve(String),

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Invalid(_) => "INVALID",
            AppError::Resolve(_) => "RESOLVE_FAILED",
            AppError::Internal(_) => "INTERNAL",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
