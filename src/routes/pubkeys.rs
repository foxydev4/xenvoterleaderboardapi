use warp::Filter;
use mongodb::{Collection, bson::{doc, Document}};
use warp::reply::{json, with_status};
use serde_json::json;
use log::{error};
use std::collections::HashMap;
use futures_util::StreamExt;
use crate::models::Block;

pub fn get_all_pubkey_counts(
    collection: Collection<Document>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("pubkeys")
        .and(warp::get())
        .and(with_collection(collection))
        .and_then(handle_get_all_pubkey_counts)
}

fn with_collection(
    collection: Collection<Document>,
) -> impl Filter<Extract = (Collection<Document>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || collection.clone())
}

async fn handle_get_all_pubkey_counts(
    collection: Collection<Document>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let filter = doc! {};
    match collection.find(filter, None).await {
        Ok(mut cursor) => {
            let mut pubkey_counts: HashMap<String, u32> = HashMap::new();
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
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
