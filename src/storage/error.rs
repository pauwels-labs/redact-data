use std::error::Error;
use std::fmt::Debug;
use std::fmt::Display;
use crate::CacheError;

/// Error type that converts to a warp::Rejection
#[derive(Debug)]
pub enum DataStorerError {
    /// Represents an error which occurred while interacting with the cache
    CacheError {
        source: CacheError,
    },

    /// Indicates an error which occured while interacting with the storage
    StorageError {
        source: StorageError
    },
}

impl Error for DataStorerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            DataStorerError::CacheError { ref source } => Some(source),
            DataStorerError::StorageError { ref source } => Some(source)
        }
    }
}

impl Display for DataStorerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DataStorerError::CacheError { .. } => {
                // TODO: display source error
                write!(f, "Cache error")
            }
            DataStorerError::StorageError { .. } => {
                // TODO: display source error
                write!(f, "Storage error")
            }
        }
    }
}

impl From<CacheError> for DataStorerError {
    fn from(e: CacheError) -> DataStorerError {
        DataStorerError::CacheError {
            source: e
        }
    }
}

/// Error type that converts to a warp::Rejection
#[derive(Debug)]
pub enum StorageError {
    /// Represents an error which occurred while loading a session from
    /// the backing session store.
    InternalError {
        source: Box<dyn Error + Send + Sync>,
    },

    /// Indicates the requested data was not found
    NotFound,
}

impl Error for StorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            StorageError::InternalError { ref source } => Some(source.as_ref()),
            StorageError::NotFound => None,
        }
    }
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            StorageError::InternalError { .. } => {
                write!(f, "Internal error occurred")
            }
            StorageError::NotFound { .. } => {
                write!(f, "Data not found")
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::StorageError;

    #[test]
    fn test_to_string_internal_error() {
        let s = StorageError::InternalError {
            source: Box::new(StorageError::NotFound),
        }
        .to_string();
        assert_eq!(s, "Internal error occurred");
    }

    #[test]
    fn test_to_string_not_found() {
        let s = StorageError::NotFound.to_string();
        assert_eq!(s, "Data not found");
    }
}
