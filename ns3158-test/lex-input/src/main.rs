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
#[derive(Serialize, Deserialize)]
struct DialogActionMessage {
    #[serde(rename = "contentType")]
    content_type: String,
    content: String,
}
#[derive(Serialize, Deserialize)]
struct DialogAction {
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "fulfillmentState")]
    fulfillment_state: String,
    message: DialogActionMessage,
}
#[derive(Serialize, Deserialize)]
struct CustomOutput {
    #[serde(rename = "sessionAttributes")]
    session_attributes: std::collections::HashMap<String, String>,
    #[serde(rename = "dialogAction")]
    dialog_action: DialogAction,
}
#[derive(Serialize, Deserialize)]
struct Slots {
    location: String,
    date: String,
    time: String,
    cuisine: String,
    numpeople: String,
    phonenumber: String,
}
#[derive(Serialize, Deserialize)]
struct CurrentIntent {
    slots: Slots,
}
#[derive(Serialize, Deserialize)]
struct CustomInput {
    #[serde(rename = "currentIntent")]
    current_intent: CurrentIntent,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::max());
    lambda!(my_outer_handler);

    Ok(())
}

async fn my_handler(e: serde_json::Value, _c: Context) -> Result<CustomOutput, HandlerError> {
    let e: CustomInput = serde_json::from_value(e).unwrap();
    let request = RestaurantRequest {
        phonenumber: e.current_intent.slots.phonenumber,
        cuisine: e.current_intent.slots.cuisine,
        num_people: e.current_intent.slots.numpeople,
        date_and_time: format!(
            "{} on {}",
            e.current_intent.slots.date, e.current_intent.slots.time
        ),
    };
    let sqs = rusoto_sqs::SqsClient::new(Region::UsEast1);
    sqs.send_message(SendMessageRequest {
        message_body: serde_json::to_string(&request).unwrap(),
        queue_url: "https://sqs.us-east-1.amazonaws.com/217015071650/yelp-restaurant-request"
            .to_owned(),
        ..SendMessageRequest::default()
    })
    .await
    .unwrap();
    Ok(CustomOutput {
        session_attributes: std::collections::HashMap::new(),
        dialog_action: DialogAction {
            type_: "Close".to_owned(),
            fulfillment_state: "Fulfilled".to_owned(),
            message: DialogActionMessage {
                content_type: "PlainText".to_owned(),
                content: "You’re all set. Expect my suggestions shortly! Have a good day."
                    .to_owned(),
            },
        },
    })
}

fn my_outer_handler(e: serde_json::Value, c: Context) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e, c))
}
