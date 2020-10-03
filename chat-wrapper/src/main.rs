#![feature(async_closure)]
use dynomite::FromAttributes;
use elasticsearch::{
    auth::Credentials,
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url,
    },
    Elasticsearch, IndexParts,
};
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{self};
use rusoto_core::{region::Region, request::DispatchSignedRequest};
use rusoto_credential::ProvideAwsCredentials;
use rusoto_sqs::{SendMessageRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use shared_types::{RestaurantRequest, YelpBusiness, YelpBusinessEs};
use simple_logger;
use std::error::Error;
use rusoto_lex_runtime::{LexRuntimeClient, LexRuntime, PostTextRequest, PostTextResponse};
#[derive(Serialize, Deserialize)]
struct UnstructuredMessage {
    id: String,
    text: String,
    timestamp: Option<String>,

}
#[derive(Serialize, Deserialize)]
struct Message {
    #[serde(rename = "type")]
    type_: String,
    unstructured: UnstructuredMessage
}
#[derive(Serialize, Deserialize)]
struct CustomOutput {
    messages: Vec<Message>
}
#[derive(Serialize, Deserialize)]
struct CustomInput {
    messages: Vec<Message>
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::max());
    lambda!(my_outer_handler);

    Ok(())
}

async fn my_handler(e: CustomInput, _c: Context) -> Result<CustomOutput, HandlerError> {
    let lex = LexRuntimeClient::new(Region::UsEast1);
    let response = lex.post_text(
        PostTextRequest {
            bot_alias: "aws_class_bot".to_owned(),
            bot_name: "aws_class_bot".to_owned(),
            input_text: e.messages.first().unwrap().unstructured.text.to_owned(),
            user_id: e.messages.first().unwrap().unstructured.id.to_owned(),
            ..PostTextRequest::default()
        }
    ).await.unwrap().message.unwrap_or("".to_owned());
    Ok(CustomOutput {
        messages: vec![
            Message {
                type_: "unstructured".to_owned(),
                unstructured: UnstructuredMessage {
                    id: "".to_owned(),
                    text: response,
                    timestamp: None
                }
            }
        ]
    })
}

fn my_outer_handler(e: CustomInput, c: Context) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e, c))
}
