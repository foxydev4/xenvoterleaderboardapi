use serde::{Deserialize, Serialize};
use mongodb::bson::{Document, doc};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Block {
    #[serde(rename = "blockId")]
    pub block_id: u32,
    #[serde(rename = "entries")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Entry {
    #[serde(rename = "blockId")]
    pub block_id: String,
    #[serde(rename = "finalHashes")]
    pub final_hashes: Vec<FinalHash>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FinalHash {
    #[serde(rename = "finalHash")]
    pub final_hash: String,
    #[serde(rename = "count")]
    pub count: u32,
    #[serde(rename = "pubkeys")]
    pub pubkeys: Vec<String>,
}

impl Block {
    // Convert Block struct to a MongoDB Document
    pub fn to_document(&self) -> Document {
        let mut doc = Document::new();
        doc.insert("blockId", self.block_id);
        let entries: Vec<Document> = self.entries.iter().map(|e| e.to_document()).collect();
        doc.insert("entries", entries);
        doc
    }
}

impl Entry {
    // Convert Entry struct to a MongoDB Document
    pub fn to_document(&self) -> Document {
        let mut doc = Document::new();
        doc.insert("blockId", &self.block_id);
        let final_hashes: Vec<Document> = self.final_hashes.iter().map(|f| f.to_document()).collect();
        doc.insert("finalHashes", final_hashes);
        doc
    }
}

impl FinalHash {
    // Convert FinalHash struct to a MongoDB Document
    pub fn to_document(&self) -> Document {
        let mut doc = Document::new();
        doc.insert("finalHash", &self.final_hash);
        doc.insert("count", self.count);
        doc.insert("pubkeys", &self.pubkeys);
        doc
    }
}
