
use dynomite::{
    Attributes, Item,
};
use serde::{Deserialize,Serialize};
#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
pub struct YelpCategory {
    pub alias: String,
   pub title: String,
}
#[derive(Item, Deserialize, Serialize, Debug, Clone)]
pub struct YelpBusiness {
    #[dynomite(partition_key)]
    pub id: String,
    alias: String,
    name: String,
    image_url: String,
    url: String,
   pub categories: Vec<YelpCategory>,
    rating: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct YelpBusinessEs {
    pub id: String,
    pub dynamo_id: String,
    pub cuisines: Vec<String>,
}
