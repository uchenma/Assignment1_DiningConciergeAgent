use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct CuisineIndexRequest {
    pub cuisine: String
}

#[derive(Serialize, Deserialize)]
pub struct CuisineIndexRequestChunk {
    pub cuisine: String,
    pub limit: usize,
    pub offset: u32,
}
