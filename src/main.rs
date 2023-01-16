use server::{game_server::GameServer, GameService};
use tonic::transport::Server;

mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tonic::include_file_descriptor_set!("descriptor"))
        .build()
        .unwrap();

    Server::builder()
        .add_service(GameServer::new(GameService::new()))
        .add_service(reflection_service)
        .serve("[::1]:50051".parse().unwrap())
        .await?;

    Ok(())
}
