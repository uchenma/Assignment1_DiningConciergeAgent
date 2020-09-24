use serde::Deserialize;

const API_KEY: &'static str = "";
#[derive(Deserialize, Debug)]
struct YelpBusiness {
    id: String,
    name: String,
}

#[derive(Deserialize, Debug)]
struct YelpResponse {
    businesses: Vec<YelpBusiness>,
}

async fn request_data_for_category(category: &&str) -> std::result::Result<YelpResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let request_url = format!("https://api.yelp.com/v3/businesses/search?location=NYC&categories={}", category);
    let yelp_response = client.get(&request_url).header("Authorization", format!("Bearer {}", API_KEY)).send().await?.json::<YelpResponse>().await?;
    Ok(yelp_response)
}

#[tokio::main]
async fn main() {
    let cuisines = vec!["vietnamese", "french", "italion", "chinese", "german", "spanish"];
    let data = futures::future::join_all(cuisines.iter().map(request_data_for_category)).await;
    println!("Hello, world! {:?}", data);
}
