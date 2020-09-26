#![feature(async_closure)]
use dynomite::{
    dynamodb::{
        BatchWriteItemError, BatchWriteItemInput, BatchWriteItemOutput, DynamoDb, DynamoDbClient,
        PutRequest, WriteRequest,
    },
    Attributes, Item,
};
use itertools::Itertools;
use serde::Deserialize;
use std::time::Duration;
const YELP_SEARCH_LIMIT: usize = 50;
const YELP_ITEMS_PER_CUISINE: u32 = 1000;
#[derive(Attributes, Deserialize, Debug, Clone)]
struct YelpCategory {
    alias: String,
    title: String,
}
#[derive(Item, Deserialize, Debug, Clone)]
struct YelpBusiness {
    #[dynomite(partition_key)]
    id: String,
    alias: String,
    name: String,
    image_url: String,
    url: String,
    categories: Vec<YelpCategory>,
    rating: f32,
}

#[derive(Deserialize, Debug, Clone)]
struct YelpResponse {
    businesses: Vec<YelpBusiness>,
}
#[derive(Debug, Clone)]
struct YelpData {
    businesses: Vec<YelpBusiness>,
    cuisine: String,
}

async fn request_page_for_category(
    category: &&str,
    offset: u32,
    ratelimit: std::rc::Rc<futures::lock::Mutex<ratelimit::Limiter>>,
) -> std::result::Result<Vec<YelpBusiness>, reqwest::Error> {
    let api_key = std::env::var("YELP_API_KEY").unwrap();
    let client = reqwest::Client::new();
    let request_url = format!(
        "https://api.yelp.com/v3/businesses/search?location=NYC&categories={}&limit={}&offset={}",
        category, YELP_SEARCH_LIMIT, offset
    );
     ratelimit.lock().await.wait();
    let yelp_response = client
        .get(&request_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;
    let yelp_response = yelp_response.json::<YelpResponse>().await?;
    Ok(yelp_response.businesses)
}
async fn request_data_for_category(
    category: &&str,
    ratelimit: std::rc::Rc<futures::lock::Mutex<ratelimit::Limiter>>,
) -> std::result::Result<YelpData, reqwest::Error> {
    let queries: Vec<_> = (0..YELP_ITEMS_PER_CUISINE)
        .step_by(YELP_SEARCH_LIMIT)
        .map(|o| (o, ratelimit.clone()))
        .map(async move |(offset, ratelimit)| {
            request_page_for_category(category, offset.clone(), ratelimit).await
        })
        .collect();
    let vec_of_results: Vec<std::result::Result<Vec<YelpBusiness>, reqwest::Error>> =
        futures::future::join_all(queries).await;
    let vec_of_results: std::result::Result<Vec<Vec<YelpBusiness>>, reqwest::Error> =
        vec_of_results.into_iter().collect();
    let vec_of_results: Vec<Vec<YelpBusiness>> = vec_of_results?;
    Ok(YelpData {
        businesses: vec_of_results.into_iter().flatten().collect(),
        cuisine: category.to_owned().to_owned(),
    })
}

async fn insert_records_into_dynamodb(
    businesses: Vec<YelpBusiness>,
    ratelimit: std::rc::Rc<futures::lock::Mutex<ratelimit::Limiter>>,
) -> std::result::Result<BatchWriteItemOutput, rusoto_core::RusotoError<BatchWriteItemError>> {
    let creds = rusoto_credential::ProfileProvider::new().unwrap();
    let client = DynamoDbClient::new_with(
        rusoto_core::HttpClient::new().unwrap(),
        creds,
        rusoto_core::region::Region::UsEast2,
    );
     ratelimit.lock().await.wait();
    client
        .batch_write_item(BatchWriteItemInput {
            request_items: [(
                "yelp_data".to_owned(),
                businesses
                    .into_iter()
                    .map(|business| WriteRequest {
                        put_request: Some(PutRequest {
                            item: business.clone().into(),
                        }),
                        ..WriteRequest::default()
                    })
                    .collect(),
            )]
            .iter()
            .cloned()
            .collect(),
            ..BatchWriteItemInput::default()
        })
        .await
}

#[tokio::main]
async fn main() {
    let cuisines = vec![
        "vietnamese",
        "french",
        "italian",
        "chinese",
        "german",
        "spanish",
    ];
    let ratelimit = ratelimit::Builder::new()
        .capacity(2)
        .quantum(2)
        .interval(Duration::new(1, 0))
        .build();

    let ratelimit = std::rc::Rc::new(futures::lock::Mutex::new(ratelimit));
    let data = futures::future::join_all(
        cuisines
            .iter()
            .map(|category| request_data_for_category(category, ratelimit.clone())),
    )
    .await;
    let write: Vec<Result<_, _>> = futures::future::join_all(
        data.iter()
            .map(|r| {
                let businesses = r.as_ref().unwrap();
                let cuisine = businesses.cuisine.clone();
                println!(
                    "Number of items for cuisine {}: {}",
                    businesses.cuisine.clone(),
                    businesses.businesses.len()
                );
                let res: Vec<_> = businesses
                    .businesses
                    .iter()
                    .map(|business| {
                        let cuisine = cuisine.clone();
                        YelpBusiness {
                            id: format!("{}-{}", business.id, cuisine),
                            ..business.clone()
                        }
                    })
                    .collect();
                res.into_iter()
            })
            .flatten()
            .chunks(25)
            .into_iter()
            .map(|chunk| insert_records_into_dynamodb(chunk.collect(), ratelimit.clone())),
    )
    .await;
    let write_result: Result<Vec<_>, _> = write.into_iter().collect();
    write_result.unwrap();
}
