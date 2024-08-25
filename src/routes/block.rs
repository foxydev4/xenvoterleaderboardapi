use warp::Filter;
use mongodb::{Collection, bson::{doc, Document}};
use warp::reply::{json, with_status};
use serde_json::json;
use log::{error, debug};
use crate::models::Block;

pub fn get_block_by_id(
    collection: Collection<Document>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("block" / u32)
        .and(warp::get())
        .and(with_collection(collection))
        .and_then(handle_get_block_by_id)
}

fn with_collection(
    collection: Collection<Document>,
) -> impl Filter<Extract = (Collection<Document>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || collection.clone())
}

async fn handle_get_block_by_id(
    block_id: u32,
    collection: Collection<Document>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let filter = doc! { "blockId": block_id };
    match collection.find_one(filter, None).await {
        Ok(Some(document)) => {
            debug!("Found document: {:?}", document);
            match mongodb::bson::from_document::<Block>(document) {
                Ok(block) => {
                    let response = json(&block);
                    Ok(with_status(response, warp::http::StatusCode::OK))
                }
                Err(e) => {
                    error!("Error deserializing block: {:?}", e);
                    let internal_error_reply = json(&json!({"error": "Internal Server Error"}));
                    Ok(with_status(internal_error_reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR))
                }
            }
        }
        Ok(None) => {
            let not_found_reply = json(&json!({"error": "Block not found"}));
            Ok(with_status(not_found_reply, warp::http::StatusCode::NOT_FOUND))
        }
        Err(e) => {
            error!("Error querying MongoDB: {:?}", e);
            let internal_error_reply = json(&json!({"error": "Internal Server Error"}));
            Ok(with_status(internal_error_reply, warp::http::StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}
