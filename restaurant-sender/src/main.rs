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
use rusoto_sqs::{DeleteMessageRequest, ReceiveMessageRequest, Sqs, SqsClient};
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared_types::{YelpBusiness, YelpBusinessEs};
use simple_logger;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct RestaurantRequest {
    phonenumber: String,
    cuisine: String,
    num_people: String,
    date_and_time: String,
}
#[derive(Serialize, Deserialize)]
struct CustomOutput {}
#[derive(Serialize, Deserialize)]
struct CustomInput {}

mod es {
    use serde::{Deserialize, Serialize};
    use shared_types::{YelpBusiness, YelpBusinessEs};
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Hit {
        pub _source: YelpBusinessEs,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Hits {
        pub hits: Vec<Hit>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct QueryResponse {
        pub hits: Hits,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::max());
    lambda!(my_handler_outer);

    Ok(())
}

fn format_address(address: &shared_types::YelpAddress) -> String {
    format!("{}", address.address1)
}

async fn my_handler(e: CustomInput) -> Result<CustomOutput, HandlerError> {
    use futures::stream::StreamExt;
    let sqs = rusoto_sqs::SqsClient::new(Region::UsEast2);
    let sns = rusoto_sns::SnsClient::new(Region::UsEast1);
    let sqs = &sqs;
    let sns = &sns;
    let messages = sqs
        .receive_message(ReceiveMessageRequest {
            max_number_of_messages: Some(1),
            queue_url: "https://sqs.us-east-2.amazonaws.com/217015071650/yelp-restaurant-request"
                .to_owned(),
            ..ReceiveMessageRequest::default()
        })
        .await
        .unwrap();
    futures::stream::iter(messages.messages.unwrap())
        .for_each_concurrent(1, async move |message| {
            let creds = rusoto_credential::DefaultCredentialsProvider::new()
                .unwrap()
                .credentials()
                .await
                .unwrap();
            let e: RestaurantRequest = serde_json::from_str(&message.body.unwrap()).unwrap();
            let mut request = rusoto_core::signature::SignedRequest::new(
                "GET",
                "es",
                &Region::UsEast2,
                "/yelp_restaurants/_search",
            );
            request.set_content_type("application/json".to_owned());
            request.set_hostname(Some(
                "vpc-yelp-restaurants-afhintr5ppa3f4vhraxvlhmvti.us-east-2.es.amazonaws.com"
                    .to_owned(),
            ));
            let payload = json!(
        {
            "from" : 0,
            "size" : 3,
            "query": {
                "function_score": {
                    "query": {
                        "bool": {
                            "must": [{
                                "match": {
                                    "cuisines": e.cuisine
                                }
                            }]
                        }
                    },
                    "random_score": {}
                }
            }
        } );
            let payload = payload.to_string();
            println!("{}", payload);
            request.set_payload(Some(payload));
            request.set_content_md5_header();
            request.sign(&creds);
            let client = rusoto_core::request::HttpClient::new().unwrap();
            let mut response = client.dispatch(request, None).await.unwrap();
            let buffer = response.buffer().await.unwrap();
            let response = buffer.body_as_str();
            println!("{}", response);

            let es_data: es::QueryResponse = serde_json::from_str(response).unwrap();
            let hits: Vec<_> = es_data
                .hits
                .hits
                .into_iter()
                .map(|hit| hit._source)
                .collect();
            let data = futures::future::join_all(hits.iter().map(async move |record| {
                let client = rusoto_dynamodb::DynamoDbClient::new(Region::UsEast2);
                let mut keyquery = std::collections::HashMap::new();
                keyquery.insert(
                    "id".to_owned(),
                    AttributeValue {
                        s: Some(record.dynamo_id.to_owned()),
                        ..AttributeValue::default()
                    },
                );
                let result = client
                    .get_item(GetItemInput {
                        key: keyquery,
                        table_name: "yelp_data".to_string(),
                        ..GetItemInput::default()
                    })
                    .await
                    .unwrap();
                result
                    .item
                    .map(shared_types::YelpBusiness::from_attrs)
                    .unwrap()
                    .unwrap()
            }))
            .await;
            let restaurant_suggestions_text = data
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    format!(
                        "{}. {}, located at {}",
                        index,
                        item.name,
                        format_address(&item.location)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            println!("{:?}", data);
            sns.publish(PublishInput {
                phone_number: Some(e.phonenumber),
                    message: format!(
                        "Hello! Here are my {} restaurant suggestions for {}, for {}: {}. Enjoy your meal!",
                        e.cuisine, e.num_people, e.date_and_time, restaurant_suggestions_text
                    ),
                ..PublishInput::default()
            }).await.unwrap();
            sqs.delete_message(
                DeleteMessageRequest {
                    receipt_handle: message.receipt_handle.unwrap(),
                    queue_url: "https://sqs.us-east-2.amazonaws.com/217015071650/yelp-restaurant-request"
                        .to_owned(),
                    ..DeleteMessageRequest::default()
                }
            ).await.unwrap();
            // CustomOutput {
            //     phonenumber: e.phonenumber,
            //     message: format!(
            //         "Hello! Here are my {} restaurant suggestions for {}, for {}: {}. Enjoy your meal!",
            //         e.cuisine, e.num_people, e.date_and_time, restaurant_suggestions_text
            //     ),
            // }
        })
        .await;
    Ok(CustomOutput {})
}
fn my_handler_outer(e: CustomInput, _c: Context) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e))
}
