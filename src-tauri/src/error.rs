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

    /// Refresh was asked of an album with no MusicBrainz id. The frontend answers by
    /// searching and letting the person choose, then calling `releases_link`.
    #[error("this album is not linked to MusicBrainz yet")]
    NotLinked,

    /// The chosen album is already another release you track. Carries that release's id so
    /// the frontend can offer to open it.
    #[error("already in your library as {artist} – {title}")]
    AlreadyInLibrary {
        release_id: String,
        artist: String,
        title: String,
    },

    #[error("{0}")]
    Internal(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Invalid(_) => "INVALID",
            AppError::Resolve(_) => "RESOLVE_FAILED",
            AppError::NotLinked => "NOT_LINKED",
            AppError::AlreadyInLibrary { .. } => "ALREADY_IN_LIBRARY",
            AppError::Internal(_) => "INTERNAL",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let len = if matches!(self, AppError::AlreadyInLibrary { .. }) { 3 } else { 2 };
        let mut st = s.serialize_struct("AppError", len)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        if let AppError::AlreadyInLibrary { release_id, .. } = self {
            st.serialize_field("releaseId", release_id)?;
        }
        st.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_duplicate_carries_the_release_to_open() {
        let e = AppError::AlreadyInLibrary {
            release_id: "r1".into(),
            artist: "Slint".into(),
            title: "Spiderland".into(),
        };
        assert_eq!(
            serde_json::to_value(&e).unwrap(),
            serde_json::json!({
                "code": "ALREADY_IN_LIBRARY",
                "message": "already in your library as Slint – Spiderland",
                "releaseId": "r1"
            })
        );
    }

    #[test]
    fn other_errors_stay_code_and_message() {
        assert_eq!(
            serde_json::to_value(AppError::NotLinked).unwrap(),
            serde_json::json!({ "code": "NOT_LINKED", "message": "this album is not linked to MusicBrainz yet" })
        );
    }
}
