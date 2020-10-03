
use dynomite::{
    Attributes, Item,
};
use serde::{Deserialize,Serialize};
#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
pub struct YelpCategory {
    pub alias: String,
   pub title: String,
}
#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
pub struct YelpAddress {
    // city: String,
    // country: String,
    pub address1: String,
    // address2: String,
    // address3: String,
    // state: String,
    // zip_code: String,
}
#[derive(Item, Deserialize, Serialize, Debug, Clone)]
pub struct YelpBusiness {
    #[dynomite(partition_key)]
    pub id: String,
    alias: String,
    pub name: String,
    image_url: String,
    url: String,
    pub categories: Vec<YelpCategory>,
    rating: f32,
    pub location: YelpAddress,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct YelpBusinessEs {
    pub id: String,
    pub dynamo_id: String,
    pub cuisines: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RestaurantRequest {
    pub phonenumber: String,
    pub cuisine: String,
    pub num_people: String,
    pub date_and_time: String,
}
