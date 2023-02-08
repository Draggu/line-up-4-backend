use crate::server::{GameSettings, Player, UserMove};
use bimap::BiHashMap;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    error::Error,
    Client, Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    errors::{JoinError, TryMoveError},
    vectorize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Matrix {
    #[serde(rename = "_id")]
    id: ObjectId,
    horizontal_size: u32,
    vertical_size: u32,
    is_horizontal_cyclic: bool,
    moves: HashMap<u32, Vec<Player>>,
    #[serde(with = "vectorize")]
    player_ids: BiHashMap<ObjectId, Player>,
    current_move_player: Player,
    is_finished: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    _lock: Option<ObjectId>,
}

impl Matrix {
    pub fn new(settings: &GameSettings) -> Self {
        Self {
            id: ObjectId::new(),
            horizontal_size: settings.horizontal_size,
            vertical_size: settings.vertical_size,
            is_horizontal_cyclic: settings.is_horizontal_cyclic,
            moves: HashMap::new(),
            player_ids: BiHashMap::new(),
            current_move_player: Player::P1,
            is_finished: false,
            _lock: None,
        }
    }

    pub async fn insert(&self, db: &Database) -> Result<ObjectId, Error> {
        Self::collection(db).insert_one(self, None).await?;

        Ok(self.id)
    }

    pub fn join(&mut self) -> Result<(ObjectId, Player), JoinError> {
        let new_player = *[Player::P1, Player::P2]
            .iter()
            .find_map(|player| {
                if self.player_ids.contains_right(player) {
                    None
                } else {
                    Some(player)
                }
            })
            .ok_or(JoinError::GameAlreadyFull)?;

        let id = ObjectId::new();

        self.player_ids.insert(id.clone(), new_player);

        Ok((id, new_player))
    }

    // returns value indicating if game has ended
    pub fn try_move(
        &mut self,
        UserMove { x, identity_token }: &UserMove,
    ) -> Result<(bool, Player), TryMoveError> {
        let player = *self
            .player_ids
            .get_by_left(
                &ObjectId::parse_str(&identity_token).map_err(|_| TryMoveError::IdWrongFormat)?,
            )
            .unwrap();

        if self.current_move_player != player {
            return Err(TryMoveError::OtherPlayerTurn);
        }

        if x >= &self.horizontal_size {
            return Err(TryMoveError::MissingColumn);
        }

        let column = self.moves.entry(*x).or_insert_with(Vec::new);

        if column.len() == usize::try_from(self.vertical_size).unwrap() {
            return Err(TryMoveError::ColumnFull);
        }

        column.push(player);

        self.current_move_player = match player {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        };

        self.is_finished = self.is_finished(*x);

        Ok((self.is_finished, player))
    }

    fn is_finished(&self, _last_move: u32) -> bool {
        unimplemented!();
    }

    #[inline]
    pub fn collection(db: &Database) -> Collection<Self> {
        db.collection("games")
    }

    #[inline]
    pub fn filter_by_id(id: &ObjectId) -> Document {
        doc! {
            "_id": id
        }
    }

    #[inline]
    pub fn filter_by_token(id: &ObjectId) -> Document {
        doc! {
            "player_ids": {
                "$elemMatch": {
                    "0": id
                }
            }
        }
    }

    // Err(None) means not found
    pub async fn get_and_save<T>(
        filter: Document,
        db: &Database,
        client: &Client,
        mutator: impl Fn(&mut Self) -> T,
    ) -> Result<T, Option<Error>> {
        let mut session = client
            .start_session(None)
            .await
            .expect("mongodb session to be supported");

        session.start_transaction(None).await?;

        let collection = Self::collection(db);

        let mut matrix = collection
            .find_one_and_update_with_session(
                filter.clone(),
                doc! {
                    "$set": {
                        "_lock": Some(ObjectId::new())
                    }
                },
                None,
                &mut session,
            )
            .await?
            .ok_or(None)?;

        let result = mutator(&mut matrix);

        collection
            .replace_one_with_session(filter, &matrix, None, &mut session)
            .await?;

        session.commit_transaction().await?;

        Ok(result)
    }
}
