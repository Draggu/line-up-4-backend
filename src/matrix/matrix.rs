use crate::server::{GameSettings, Player, UserMove};
use bimap::BiHashMap;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    error::Error,
    Client, Collection, Database,
};
use serde::{Deserialize, Serialize};

use super::{
    errors::{JoinError, TryMoveError},
    index_range::{Horizontal, IndexRange, Vertical},
    vectorize,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Matrix {
    #[serde(rename = "_id")]
    id: ObjectId,
    horizontal_size: u16,
    vertical_size: u16,
    is_horizontal_cyclic: bool,
    moves: Vec<Vec<Player>>,
    #[serde(with = "vectorize")]
    player_ids: BiHashMap<ObjectId, Player>,
    current_move_player: Player,
    is_finished: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    _lock: Option<ObjectId>,
}

impl Matrix {
    const LINE: usize = 4;
    const LINE_WITHOUT_FIRST: usize = Self::LINE - 1;

    /// * `ir` is start point and will not be counted
    fn length_in_dir(&self, mut ir: IndexRange, v: Vertical, h: Option<Horizontal>) -> usize {
        let first_column = &self.moves[ir.as_usize()];
        let y = first_column.len() - 1;

        // there must be minimum 1 element
        let last_player = first_column.last().unwrap();

        (1..Self::LINE)
            .take_while(|i| {
                // if no horizontal move then do not move cursor
                h.map(|h| ir.move_cursor(1, h)).unwrap_or(true)
                    && self
                        .moves
                        .get(ir.as_usize())
                        .unwrap()
                        .get(match v {
                            Vertical::Up => y + i,
                            Vertical::Down => y - i,
                            Vertical::Straight => y,
                        })
                        .filter(|player| *player == last_player)
                        .is_some()
            })
            .count()
    }

    fn is_finished(&self, last_move: u16) -> bool {
        let ir = IndexRange::new(self.horizontal_size, self.is_horizontal_cyclic, last_move);

        macro_rules! line {
            ($v1:expr,$v2:expr,$h1:expr,$h2:expr) => {{
                let one_side_length = self.length_in_dir(ir, $v1, $h1);

                one_side_length == Self::LINE_WITHOUT_FIRST
                    || (one_side_length + self.length_in_dir(ir, $v2, $h2))
                        == Self::LINE_WITHOUT_FIRST
            }};
        }

        return line! {
            Vertical::Down,Vertical::Up,None,None
        } || line! {
            Vertical::Down,Vertical::Up,Some(Horizontal::Left),Some(Horizontal::Right)
        } || line! {
            Vertical::Up,Vertical::Down,Some(Horizontal::Left),Some(Horizontal::Right)
        } || line! {
            Vertical::Straight,Vertical::Straight,Some(Horizontal::Left),Some(Horizontal::Right)
        };
    }

    pub fn new(settings: &GameSettings) -> Self {
        Self {
            id: ObjectId::new(),
            horizontal_size: settings.horizontal_size as u16,
            vertical_size: settings.vertical_size as u16,
            is_horizontal_cyclic: settings.is_horizontal_cyclic,
            moves: vec![Vec::new(); settings.horizontal_size as usize],
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

        let x = *x as u16;

        if x >= self.horizontal_size.into() {
            return Err(TryMoveError::MissingColumn);
        }

        // at this point x is always in range
        let column = self.moves.get_mut(x as usize).unwrap();

        if column.len() == self.vertical_size as usize {
            return Err(TryMoveError::ColumnFull);
        }

        column.push(player);

        self.current_move_player = match player {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        };

        self.is_finished = self.is_finished(x);

        Ok((self.is_finished, player))
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
