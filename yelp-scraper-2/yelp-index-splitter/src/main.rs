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
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{self};
use rusoto_core::{region::Region, request::DispatchSignedRequest};
use rusoto_credential::ProvideAwsCredentials;
use rusoto_dynamodb::{AttributeValue, DynamoDb, GetItemInput};
use rusoto_sns::{PublishInput, Sns, SnsClient};
use rusoto_sqs::{DeleteMessageRequest, SendMessageRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_logger;
use std::error::Error;
const YELP_SEARCH_LIMIT: usize = 50;
const YELP_ITEMS_PER_CUISINE: u32 = 1000;
#[derive(Serialize, Deserialize)]
struct CustomOutput {}
#[derive(Serialize, Deserialize)]
struct CustomInput {}


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
fn my_handler_outer(e: LambdaSqsInput, _c: Context) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e))
}

async fn my_handler(e:LambdaSqsInput) -> Result<CustomOutput, HandlerError> {
    let e: yelp_shared_types::CuisineIndexRequest = e.records.iter().map(|record|serde_json::from_str(&record.body).unwrap()).next().unwrap();
    let sqs = rusoto_sqs::SqsClient::new(Region::UsEast1);
    let sqs = &sqs;
    let queries: Vec<_> = (0..YELP_ITEMS_PER_CUISINE)
        .step_by(YELP_SEARCH_LIMIT)
        .map(|offset| yelp_shared_types::CuisineIndexRequestChunk {cuisine: e.cuisine.to_owned(), offset, limit: YELP_SEARCH_LIMIT})
        .map(|request|
             sqs
             .send_message(SendMessageRequest {
                 message_body: serde_json::to_string(&request).unwrap(),
                 queue_url: "https://sqs.us-east-1.amazonaws.com/217015071650/yelp-index-chunked".to_owned(),
                 ..SendMessageRequest::default()
             })
        )
        .collect::<Vec<_>>();
    futures::future::join_all(queries).await;
    Ok(CustomOutput{})
}
