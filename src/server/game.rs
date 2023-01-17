use game_server::Game;
use mongodb::{bson::Bson, Collection, Database};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status};

use crate::matrix::Matrix;

tonic::include_proto!("main");

pub struct GameService {
    collection: Collection<Matrix>,
}

impl GameService {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection::<Matrix>("games"),
        }
    }
}

#[tonic::async_trait]
impl Game for GameService {
    type moveStream = ReceiverStream<Result<MoveInfo, Status>>;

    async fn create(&self, request: Request<GameSettings>) -> Result<Response<GameId>, Status> {
        let err = || Status::new(Code::Internal, "Internal Server Error");

        if let Bson::ObjectId(game) = self
            .collection
            .insert_one(Matrix::new(request.get_ref()), None)
            .await
            .map_err(|_| err())
            .map(|r| r.inserted_id)?
        {
            Ok(Response::new(GameId { id: game.to_hex() }))
        } else {
            Err(err())
        }
    }

    async fn join(&self, request: Request<GameId>) -> Result<Response<PlayerAssigment>, Status> {
        unimplemented!();
    }

    async fn r#move(
        &self,
        request: Request<tonic::Streaming<Move>>,
    ) -> Result<Response<Self::moveStream>, Status> {
        unimplemented!();
    }
}
