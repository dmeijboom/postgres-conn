use crate::backend::State;
use crate::proto::messages::{ErrorResponse, PasswordMessage, Severity};

#[derive(Debug, PartialEq)]
pub enum AuthMethod {
    CleartextPassword,
    None,
}

pub type AuthResult = Result<(), ErrorResponse>;

pub trait Auth {
    fn method(&self, state: &State) -> AuthMethod;
    fn clear_text_password(&self, _state: &State, _password: PasswordMessage) -> AuthResult {
        Err(ErrorResponse::new(
            Severity::Error,
            "XX000".to_string(),
            "cleartext password not supported".to_string(),
        ))
    }
}

pub struct NoopAuth {}

impl NoopAuth {
    pub fn new() -> Self {
        Self {}
    }
}

impl Auth for NoopAuth {
    fn method(&self, _state: &State) -> AuthMethod {
        AuthMethod::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_auth() {
        assert_eq!(NoopAuth::new().method(&State::default()), AuthMethod::None);
    }
}
