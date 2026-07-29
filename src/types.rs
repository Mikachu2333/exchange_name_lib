use std::{fmt, io, path::PathBuf};

/// Renaming failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameError {
    PermissionDenied,
    AlreadyExists,
    NotExists,
    SamePath,
    InvalidPath(String),
    UnsupportedFileType(PathBuf),
    /// The requested operation failed and at least one rollback operation also failed.
    /// The filesystem may require manual recovery.
    RollbackFailed {
        operation: String,
        rollback: String,
    },
    Unknown(String),
}

impl RenameError {
    /// Map errors to stable C API return codes.
    #[must_use]
    pub const fn to_code(&self) -> i32 {
        match self {
            Self::NotExists => 1,
            Self::PermissionDenied => 2,
            Self::AlreadyExists => 3,
            Self::SamePath => 4,
            Self::InvalidPath(_) => 5,
            Self::UnsupportedFileType(_) => 6,
            Self::RollbackFailed { .. } => 7,
            Self::Unknown(_) => 255,
        }
    }
}

impl fmt::Display for RenameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => f.write_str("permission denied"),
            Self::AlreadyExists => f.write_str("target already exists"),
            Self::NotExists => f.write_str("path does not exist"),
            Self::SamePath => f.write_str("the two paths refer to the same file"),
            Self::InvalidPath(message) => write!(f, "invalid path: {message}"),
            Self::UnsupportedFileType(path) => {
                write!(f, "unsupported file type: {}", path.display())
            }
            Self::RollbackFailed {
                operation,
                rollback,
            } => write!(
                f,
                "rename failed ({operation}) and rollback also failed ({rollback}); filesystem state may be inconsistent"
            ),
            Self::Unknown(message) => write!(f, "unknown error: {message}"),
        }
    }
}

impl std::error::Error for RenameError {}

impl From<io::Error> for RenameError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            io::ErrorKind::NotFound => Self::NotExists,
            io::ErrorKind::PermissionDenied | io::ErrorKind::ReadOnlyFilesystem => {
                Self::PermissionDenied
            }
            io::ErrorKind::AlreadyExists | io::ErrorKind::DirectoryNotEmpty => Self::AlreadyExists,
            io::ErrorKind::InvalidInput
            | io::ErrorKind::InvalidFilename
            | io::ErrorKind::NotADirectory => Self::InvalidPath(value.to_string()),
            io::ErrorKind::CrossesDevices => {
                Self::Unknown(format!("cross-device rename is not supported: {value}"))
            }
            _ => Self::Unknown(value.to_string()),
        }
    }
}
