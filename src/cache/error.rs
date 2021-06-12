use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;

/// Error type that converts to a warp::Rejection
#[derive(Debug)]
pub enum CacheError {
    /// Represents an error which occurred while retrieving the data from the cache
    InternalError {
        source: Box<dyn Error + Send + Sync>,
    },

    /// Indicates the requested data was not found
    NotFound,
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            CacheError::InternalError { ref source } => Some(source.as_ref()),
            CacheError::NotFound => None,
        }
    }
}

impl Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            CacheError::InternalError { .. } => {
                write!(f, "Internal error occurred")
            }
            CacheError::NotFound { .. } => {
                write!(f, "Cache entry not found")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::CacheError;

    #[test]
    fn test_to_string_internal_error() {
        let s = CacheError::InternalError {
            source: Box::new(CacheError::NotFound),
        }
            .to_string();
        assert_eq!(s, "Internal error occurred");
    }

    #[test]
    fn test_to_string_not_found() {
        let s = CacheError::NotFound.to_string();
        assert_eq!(s, "Cache entry not found");
    }
}
