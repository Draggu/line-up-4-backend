use db::db::create_db;
use dotenv::dotenv;
use server::{game_server::GameServer, GameService};
use tonic::transport::Server;

mod db;
mod matrix;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let (database, client) = create_db().await?;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tonic::include_file_descriptor_set!("descriptor"))
        .build()
        .unwrap();

    Server::builder()
        .add_service(GameServer::new(GameService::new(database, client)))
        .add_service(reflection_service)
        .serve(
            std::env::var("SERVER_URL")
                .unwrap_or("[::1]:50051".to_string())
                .parse()
                .unwrap(),
        )
        .await?;

    Ok(())
}
