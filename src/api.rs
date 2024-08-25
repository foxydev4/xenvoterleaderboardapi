use serde::{Deserialize, Serialize};
use reqwest::Error;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct FinalHash {
    pub finalHash: String,
    pub count: u32,
    pub pubkeys: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub blockId: String,
    pub finalHashes: Vec<FinalHash>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub blockId: u64,
    pub entries: Vec<Entry>,
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("Network request failed")]
    RequestError(#[from] reqwest::Error),
    #[error("Server returned a non-success status: {0}")]
    NonSuccessStatus(reqwest::StatusCode),
}

pub async fn fetch_data(api_endpoint: &str, block_id: u64) -> Result<ApiResponse, FetchError> {
    let url = format!("{}/{}", api_endpoint, block_id);
    let response = reqwest::get(&url).await?;

    if response.status().is_success() {
        Ok(response.json::<ApiResponse>().await?)
    } else {
        Err(FetchError::NonSuccessStatus(response.status()))
    }
}
