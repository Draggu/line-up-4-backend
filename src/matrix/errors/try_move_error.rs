#[derive(Debug)]
pub enum TryMoveError {
    ColumnFull,
    MissingColumn,
    OtherPlayerTurn,
    IdWrongFormat,
}

impl Into<&'static str> for TryMoveError {
    fn into(self) -> &'static str {
        match self {
            TryMoveError::OtherPlayerTurn => "not your move",
            TryMoveError::MissingColumn => "column does not exists",
            TryMoveError::ColumnFull => "column already filled",
            TryMoveError::IdWrongFormat => "id should be 24-char hexadecimal string",
        }
    }
}
