use crate::backend::State;
use crate::proto::messages::PasswordMessage;

#[derive(Debug, PartialEq)]
pub enum AuthMethod {
    CleartextPassword,
    None,
}

pub enum AuthResult {
    Ok,
    Err(String),
}

pub trait Auth {
    fn method(&self, state: &State) -> AuthMethod;
    fn clear_text_password(&self, _password: PasswordMessage) -> AuthResult {
        AuthResult::Err("Not implemented".to_string())
    }
}

pub struct NoneAuth {}

impl NoneAuth {
    pub fn new() -> Self {
        Self {}
    }
}

impl Auth for NoneAuth {
    fn method(&self, _state: &State) -> AuthMethod {
        AuthMethod::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_auth() {
        assert_eq!(NoneAuth::new().method(&State::default()), AuthMethod::None);
    }
}
