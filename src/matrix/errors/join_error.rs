#[derive(Debug)]
pub enum JoinError {
    GameAlreadyFull,
}

impl Into<&'static str> for JoinError {
    fn into(self) -> &'static str {
        match self {
            JoinError::GameAlreadyFull => "game is already full",
        }
    }
}
