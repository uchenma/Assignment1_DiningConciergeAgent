use serde::Deserialize;
use rusoto_dynamodb::DynamoDb;
use rusoto_credential::ProvideAwsCredentials;

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
    let api_key = std::env::var("YELP_API_KEY").unwrap();
    let client = reqwest::Client::new();
    let request_url = format!("https://api.yelp.com/v3/businesses/search?location=NYC&categories={}", category);
    let yelp_response = client.get(&request_url).header("Authorization", format!("Bearer {}", api_key)).send().await?.json::<YelpResponse>().await?;
    Ok(yelp_response)
}

#[tokio::main]
async fn main() {
    let cuisines = vec!["vietnamese"/*, "french", "italion", "chinese", "german", "spanish"*/];
    let data = futures::future::join_all(cuisines.iter().map(request_data_for_category)).await;
    let creds =rusoto_credential::ProfileProvider::new().unwrap();
    let client =  rusoto_dynamodb::DynamoDbClient::new_with(rusoto_core::HttpClient::new().unwrap(),creds,  rusoto_core::region::Region::UsEast2);
    client.put_item( rusoto_dynamodb::PutItemInput {
        condition_expression: Option::None,
        conditional_operator: Option::None,
        expected: Option::None,
        expression_attribute_names: Option::None,
        expression_attribute_values: Option::None,
        item: std::collections::HashMap::new(),
        return_consumed_capacity: Option::None,
        return_item_collection_metrics: Option::None,
        return_values: Option::None,
        table_name: "yelp_data".to_string(),
    }).await.unwrap();
    println!("Hello, world! {:?}", data);
}
