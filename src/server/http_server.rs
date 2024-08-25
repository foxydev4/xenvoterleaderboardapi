use warp::Filter;
use mongodb::{Collection, bson::Document};

// Import route handlers from the crate root
use crate::routes::{get_block_by_id, get_all_pubkey_counts, get_blocks_in_range};

pub async fn run_http_server(collection: Collection<Document>) {
    // Define the routes for the REST API
    let block_route = get_block_by_id(collection.clone());
    let pubkey_counts_route = get_all_pubkey_counts(collection.clone());
    let pubkey_ranges = get_blocks_in_range(collection);

    // Combine the routes
    let api_routes = block_route
        .or(pubkey_counts_route)
        .or(pubkey_ranges);

    // Serve the HTTP server on port 3031 (or another port of your choice)
    warp::serve(api_routes)
        .run(([0, 0, 0, 0], 3031))
        .await;
}
