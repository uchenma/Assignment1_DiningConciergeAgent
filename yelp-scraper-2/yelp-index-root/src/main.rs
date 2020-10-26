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
#[derive(Serialize, Deserialize)]
struct CustomOutput {}
#[derive(Serialize, Deserialize)]
struct CustomInput {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::max());
    lambda!(my_handler_outer);

    Ok(())
}
fn my_handler_outer(e: CustomInput, _c: Context) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e))
}

async fn my_handler(e: CustomInput) -> Result<CustomOutput, HandlerError> {
    let sqs = rusoto_sqs::SqsClient::new(Region::UsEast1);
    let sqs = &sqs;
    let queries = futures::future::join_all(
        (vec![
            "african",
            "american",
            "asian",
            "chinese",
            "ethiopian",
            "french",
            "german",
            "greek",
            "indian",
            "italian",
            "japanese",
            "korean",
            "mexican",
            "moroccan",
            "polish",
            "spanish",
            "thai",
            "vietnamese",
        ])
        .into_iter()
        .map(|cuisine| yelp_shared_types::CuisineIndexRequest {
            cuisine: cuisine.to_owned(),
        })
        .map(async move |request| {
            let request = serde_json::to_string(&request).unwrap();
            sqs.send_message(SendMessageRequest {
                message_body: request,
                queue_url: "https://sqs.us-east-1.amazonaws.com/217015071650/yelp-index-requests"
                    .to_owned(),
                ..SendMessageRequest::default()
            })
            .await
            .unwrap()
        })
        .collect::<Vec<_>>(),
    )
    .await;
    Ok(CustomOutput {})
}
