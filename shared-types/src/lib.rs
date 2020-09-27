
use dynomite::{
    Attributes, Item,
};
use serde::{Deserialize,Serialize};
#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
struct YelpCategory {
    alias: String,
    title: String,
}
#[derive(Item, Deserialize, Serialize, Debug, Clone)]
pub struct YelpBusiness {
    #[dynomite(partition_key)]
    pub id: String,
    alias: String,
    name: String,
    image_url: String,
    url: String,
    categories: Vec<YelpCategory>,
    rating: f32,
}
