use std::fmt::Debug;

use futures::StreamExt;
use game_server::Game;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Client, Database,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::matrix::Matrix;

tonic::include_proto!("main");

pub struct GameService {
    db: Database,
    client: Client,
}

impl GameService {
    pub fn new(db: Database, client: Client) -> Self {
        Self { db, client }
    }

    fn internal_server_err<T>(err: T) -> Status
    where
        T: Debug,
    {
        println!("{:?}", err);
        Status::internal("Internal Server Error")
    }
}

#[tonic::async_trait]
impl Game for GameService {
    type MoveStream = ReceiverStream<Result<MoveInfo, Status>>;

    async fn create(&self, request: Request<GameSettings>) -> Result<Response<GameId>, Status> {
        Ok(Response::new(GameId {
            id: Matrix::new(request.get_ref())
                .insert(&self.db)
                .await
                .map_err(Self::internal_server_err)?
                .to_hex(),
        }))
    }

    async fn join(&self, request: Request<GameId>) -> Result<Response<PlayerAssigment>, Status> {
        let GameId { id } = request.into_inner();

        let (id, player) = Matrix::get_and_save(
            Matrix::filter_by_id(&ObjectId::parse_str(id).map_err(Self::internal_server_err)?),
            &self.db,
            &self.client,
            Matrix::join,
        )
        .await
        //TODO better handle
        .map_err(Self::internal_server_err)? // db errors
        .map_err(Self::internal_server_err)?; // Matrix::join errors

        let mut assigment = PlayerAssigment::default();

        assigment.set_player(player);
        assigment.identity_token = id.to_hex();

        Ok(Response::new(assigment))
    }

    async fn r#move(
        &self,
        request: Request<tonic::Streaming<UserMove>>,
    ) -> Result<Response<Self::MoveStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(4);
        let db = self.db.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            while let Some(user_move) = stream.next().await {
                let inner = async {
                    let user_move = user_move?;

                    let (is_finished, player) = Matrix::get_and_save(
                        Matrix::filter_by_token(
                            &ObjectId::parse_str(&user_move.identity_token)
                                .map_err(Self::internal_server_err)?,
                        ),
                        &db,
                        &client,
                        |game| game.try_move(&user_move),
                    )
                    .await
                    //TODO better handle
                    .map_err(Self::internal_server_err)? // db errors
                    .map_err(Self::internal_server_err)?; // Matrix::try_move errors

                    let mut result = MoveInfo::default();

                    result.x = user_move.x;
                    result.is_last_move = is_finished;
                    result.set_player(player);

                    Ok(result)
                };

                if let Err(_) = tx.send(inner.await).await {
                    break;
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
