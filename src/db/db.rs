use mongodb::{options::ClientOptions, Client, Database};

pub async fn create_db() -> mongodb::error::Result<(Database, Client)> {
    let mut client_options =
        ClientOptions::parse(std::env::var("DB_URL").expect("DB_URL must be set")).await?;

    client_options.retry_writes = Some(false);

    let client = Client::with_options(client_options)?;

    Ok((
        client.database(
            std::env::var("DB_NAME")
                .expect("DB_NAME must be set")
                .as_str(),
        ),
        client,
    ))
}
