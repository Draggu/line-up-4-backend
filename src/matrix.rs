use crate::server::GameSettings;
use crate::server::Player;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Matrix {
    horizontal_size: u32,
    vertical_size: u32,
    is_horizontal_cyclic: bool,
    moves: HashMap<(u32, u32), Player>,
    player_ids: HashMap<Player, String>,
}

impl Matrix {
    pub fn new(settings: &GameSettings) -> Self {
        Self {
            horizontal_size: settings.horizontal_size,
            vertical_size: settings.vertical_size,
            is_horizontal_cyclic: settings.is_horizontal_cyclic,
            moves: HashMap::new(),
            player_ids: HashMap::new(),
        }
    }
}
