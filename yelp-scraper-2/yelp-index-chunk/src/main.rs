#![feature(async_closure)]
use dynomite::FromAttributes;
use elasticsearch::{
    auth::Credentials,
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url,
    },
    Elasticsearch, IndexParts, SearchParts,
};
use dynomite::dynamodb::{
    BatchWriteItemError, BatchWriteItemInput, BatchWriteItemOutput, DynamoDb, DynamoDbClient,
    PutRequest, WriteRequest,
};
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{self};
use rusoto_core::{region::Region, request::DispatchSignedRequest};
use rusoto_credential::ProvideAwsCredentials;
use rusoto_sns::{PublishInput, Sns, SnsClient};
use rusoto_sqs::{DeleteMessageRequest, SendMessageRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_logger;
use shared_types::YelpBusiness;
use std::error::Error;
use itertools::Itertools;
#[derive(Serialize, Deserialize)]
struct CustomOutput {}
#[derive(Serialize, Deserialize)]
struct CustomInput {}
#[derive(Deserialize, Debug, Clone)]
struct YelpResponse {
    businesses: Vec<YelpBusiness>,
}
#[derive(Serialize, Deserialize)]
struct LambdaSqsRecord {
    body: String
}
#[derive(Serialize, Deserialize)]
struct LambdaSqsInput {
    #[serde(rename = "Records")]
    records: Vec<LambdaSqsRecord>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::max());
    lambda!(my_handler_outer);

    Ok(())
}
fn my_handler_outer(
    e: LambdaSqsInput,
    _c: Context,
) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e))
}

async fn my_handler(
    e: LambdaSqsInput,
) -> Result<CustomOutput, HandlerError> {
    let e: yelp_shared_types::CuisineIndexRequestChunk = e.records.iter().map(|record|serde_json::from_str(&record.body).unwrap()).next().unwrap();
    let api_key = std::env::var("YELP_API_KEY").unwrap();
    let client = reqwest::Client::new();
    let request_url = format!(
        "https://api.yelp.com/v3/businesses/search?location=NYC&categories={}&limit={}&offset={}",
        e.cuisine, e.limit, e.offset
    );
    let yelp_response = client
        .get(&request_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await.unwrap();
    let response = yelp_response.text().await.unwrap();
    println!("{}", response);
    let yelp_response = serde_json::from_str::<YelpResponse>(&response).unwrap();
    let client = DynamoDbClient::new(rusoto_core::region::Region::UsEast1);
    let queries = yelp_response
        .businesses
        .chunks(25)
        .map(|businesses| {
            println!("{:?}", businesses);
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
        });
    futures::future::join_all(queries).await.into_iter().map(|r| r.unwrap()).collect::<Vec<_>>();
    Ok(CustomOutput {})
}
