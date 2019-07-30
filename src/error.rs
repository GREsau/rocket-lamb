#[derive(Debug, Fail)]
pub(crate) enum RocketLambError {
    #[fail(display = "could not transform request: {}", 0)]
    InvalidRequest(String),
    #[fail(display = "could not transform response: {}", 0)]
    InvalidResponse(String),
}

macro_rules! invalid_request {
    ($($arg:tt)*) => (RocketLambError::InvalidRequest(format!($($arg)*)))
}

macro_rules! invalid_response {
    ($($arg:tt)*) => (RocketLambError::InvalidResponse(format!($($arg)*)))
}
