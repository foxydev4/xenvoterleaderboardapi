use std::thread::sleep;
use std::time::Duration;
use reqwest;
use tokio::sync::broadcast;
use mongodb::{Collection, bson::Document};
use serde_json::Value;

use crate::models;

const START_BLOCK_ID: i32 = 27961401;
const END_BLOCK_ID: i32 = 27965401;
const BLOCK_INCREMENT: i32 = 100;

pub async fn fetch_data_and_broadcast(tx: broadcast::Sender<String>, collection: Collection<Document>) {
    let client = reqwest::Client::new();
    let mut block_id = START_BLOCK_ID; // Starting block ID

    loop {
        let url = format!("http://xolana.xen.network:4444/fetch_data/{}", block_id);

        match client.get(&url).send().await {
            Ok(response) => {
                match response.text().await {
                    Ok(body) => {
                        // Broadcast the data
                        let _ = tx.send(body.clone());

                        // Parse the JSON data
                        let json_data: Value = serde_json::from_str(&body).unwrap_or_default();
                        let block: Result<models::Block, _> = serde_json::from_value(json_data);

                        match block {
                            Ok(block) => {
                                // Save the data to MongoDB
                                let doc = block.to_document();
                                if let Err(e) = save_data_to_mongo(&collection, doc).await {
                                    eprintln!("Error saving data for block ID {}: {}", block_id, e);
                                }

                                println!("Broadcasted and saved data for block ID {}.", block_id); // Optional logging
                            }
                            Err(e) => eprintln!("Error deserializing data for block ID {}: {}", block_id, e),
                        }
                    }
                    Err(e) => eprintln!("Error reading response body for block ID {}: {}", block_id, e),
                }
            }
            Err(e) => eprintln!("Error fetching data for block ID {}: {}", block_id, e),
        }

        // Increment the block ID
        block_id += BLOCK_INCREMENT;

        // Check if block ID exceeds the end limit, reset to start
        if block_id > END_BLOCK_ID {
            block_id = START_BLOCK_ID;
        }
        sleep(Duration::from_secs(1));
    }
}

async fn save_data_to_mongo(collection: &Collection<Document>, doc: Document) -> Result<(), mongodb::error::Error> {
    collection.insert_one(doc, None).await?;
    Ok(())
}
