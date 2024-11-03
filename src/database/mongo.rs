use mongodb::{Client, Database};

pub async fn connect(uri:&str, name:&str) -> mongodb::error::Result<Database> {
    let client = Client::with_uri_str(uri).await?;
    let db = client.database(name);
    println!("Connected to {:?} in {:?}",uri, name);
    Ok(db)
}