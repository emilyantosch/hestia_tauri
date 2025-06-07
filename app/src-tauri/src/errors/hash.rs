pub struct HashError {
    pub kind: HashErrorKind,
}

pub enum HashErrorKind {
    IoError(std::io::Error),
    InvalidPathError,
    PermissionDeniedError,
}

impl From<std::io::Error> for HashErrorKind {
    fn from(other: std::io::Error) -> HashErrorKind {
        HashErrorKind::IoError(other)
    }
}
