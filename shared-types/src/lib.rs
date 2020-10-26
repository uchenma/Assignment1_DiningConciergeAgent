use dynomite::{Attributes, Item};
use serde::{Deserialize, Serialize};
#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
pub struct YelpCategory {
    pub alias: String,
    pub title: String,
}
#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
pub struct YelpAddress {
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    country: Option<String>,
    pub address1: String,
    #[serde(default)]
    address2: Option<String>,
    #[serde(default)]
    address3: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    zip_code: Option<String>,
}

#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
pub struct YelpCoordinates {
    #[serde(default)]
    latitude: Option<f32>,
    #[serde(default)]
    longitude: Option<f32>,
}
#[derive(Item, Deserialize, Serialize, Debug, Clone)]
pub struct YelpBusiness {
    #[dynomite(partition_key)]
    pub id: String,
    pub alias: String,
    pub name: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    pub categories: Vec<YelpCategory>,
    #[serde(default)]
    pub rating: Option<f32>,
    #[serde(default)]
    pub review_count: Option<u32>,
    #[serde(default)]
    pub coordinates: Option<YelpCoordinates>,
    pub location: YelpAddress,
    #[serde(default)]
    #[serde(rename(serialize = "insertedAtTimestamp"))]
    pub inserted_at_timestamp: Option<String>,
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
