use warp::Filter;
use mongodb::{Collection, bson::{doc, Document}};
use warp::reply::{json, with_status};
use serde_json::json;
use log::{error, info};
use std::collections::HashMap;
use futures_util::StreamExt;
use crate::models::{Block, Entry, FinalHash};

pub fn get_blocks_in_range(
    collection: Collection<Document>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("blocks" / i32 / i32)
        .and(warp::get())
        .and(with_collection(collection))
        .and_then(handle_get_blocks_in_range)
}

fn with_collection(
    collection: Collection<Document>,
) -> impl Filter<Extract = (Collection<Document>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || collection.clone())
}

async fn handle_get_blocks_in_range(
    start_id: i32,
    end_id: i32,
    collection: Collection<Document>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if start_id > end_id {
        return Ok(with_status(
            json(&json!({"error": "Invalid range: start_id is greater than end_id"})),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    let filter = doc! {
        "blockId": { "$gte": start_id, "$lte": end_id }
    };

    info!("Querying MongoDB with filter: {:?}", filter);

    match collection.find(filter, None).await {
        Ok(mut cursor) => {
            let mut pubkey_counts: HashMap<String, u32> = HashMap::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        info!("Fetched document: {:?}", document);
                        match mongodb::bson::from_document::<Block>(document) {
                            Ok(block) => {
                                for entry in block.entries {
                                    for pubkey in entry.final_hashes.iter().flat_map(|fh| &fh.pubkeys) {
                                        *pubkey_counts.entry(pubkey.clone()).or_insert(0) += 1;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Error deserializing block: {:?}", e);
                                let internal_error_reply = json(&json!({"error": "Internal Server Error"}));
                                return Ok(with_status(internal_error_reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error fetching document: {:?}", e);
                        let internal_error_reply = json(&json!({"error": "Internal Server Error"}));
                        return Ok(with_status(internal_error_reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR));
                    }
                }
            }

            if pubkey_counts.is_empty() {
                info!("No pubkeys found in the specified range.");
                return Ok(with_status(json(&json!({"message": "No pubkeys found"})), warp::http::StatusCode::NOT_FOUND));
            }

            // Convert the HashMap to a Vec of tuples for sorting
            let mut sorted_pubkey_counts: Vec<(String, u32)> = pubkey_counts.into_iter().collect();
            // Sort the pubkey_counts by count in descending order
            sorted_pubkey_counts.sort_by(|a, b| b.1.cmp(&a.1));

            // Convert the sorted Vec back to a JSON object
            let response = json(&sorted_pubkey_counts);
            Ok(with_status(response, warp::http::StatusCode::OK))
        }
        Err(e) => {
            error!("Error querying MongoDB: {:?}", e);
            let internal_error_reply = json(&json!({"error": "Internal Server Error"}));
            Ok(with_status(internal_error_reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}
