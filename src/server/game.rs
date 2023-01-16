use game_server::Game;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

tonic::include_proto!("main");

pub struct GameService {}

impl GameService {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl Game for GameService {
    type moveStream = ReceiverStream<Result<MoveInfo, Status>>;

    async fn create(&self, request: Request<GameSettings>) -> Result<Response<GameId>, Status> {
        let GameSettings {
            is_horizontal_cyclic,
            horizontal_size,
            vertical_size,
        } = request.get_ref();
        unimplemented!();
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
