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
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared_types::{YelpBusiness, YelpBusinessEs};
use simple_logger;
use std::error::Error;

#[derive(Serialize, Deserialize)]
struct RestaurantRequest {
    phonenumber: String,
    cuisine: String,
}
#[derive(Serialize, Deserialize)]
struct CustomOutput {
    message: String,
    phonenumber: String,
}

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

async fn my_handler(e: RestaurantRequest) -> Result<CustomOutput, HandlerError> {
    use futures::stream::StreamExt;
    let creds = rusoto_credential::DefaultCredentialsProvider::new()
        .unwrap()
        .credentials()
        .await
        .unwrap();
    let mut request = rusoto_core::signature::SignedRequest::new(
        "GET",
        "es",
        &Region::UsEast2,
        "/yelp_restaurants/_search",
    );
    request.set_content_type("application/json".to_owned());
    request.set_hostname(Some(
        "vpc-yelp-restaurants-afhintr5ppa3f4vhraxvlhmvti.us-east-2.es.amazonaws.com".to_owned(),
    ));
    let payload = json!(
        {
            "query": {
                "function_score": {
                    "query": {
                        "match" : { "cuisine" : e.cuisine }
                    },
                    "random_score": {}
                }
            }
        } );
    let payload = json!(
        {
            "query": {
                "match" : { "cuisine" : e.cuisine }
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
    let data = futures::future::join_all(
        hits.iter().map(async move |record| {
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
            result.item.map(shared_types::YelpBusiness::from_attrs)
        })
    )
    .await;
    println!("{:?}", data);
    Ok(CustomOutput {
        phonenumber: e.phonenumber,
        message: "Hi".to_owned(),
    })
}
fn my_handler_outer(e: RestaurantRequest, _c: Context) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e))
}
