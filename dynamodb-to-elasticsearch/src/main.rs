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
use serde::Serialize;
use shared_types::{YelpBusiness, YelpBusinessEs};
use simple_logger;
use std::error::Error;

#[derive(Serialize)]
struct CustomOutput {}

fn to_dynomite(attr: rusoto_dynamodbstreams::AttributeValue) -> dynomite::AttributeValue {
    dynomite::AttributeValue {
        b: attr.b,
        bool: attr.bool,
        bs: attr.bs,
        l: attr.l.map(|v| v.into_iter().map(to_dynomite).collect()),
        m: attr
            .m
            .map(|m| m.into_iter().map(|(k, v)| (k, to_dynomite(v))).collect()),
        n: attr.n,
        ns: attr.ns,
        null: attr.null,
        s: attr.s,
        ss: attr.ss,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::max());
    lambda!(my_outer_handler);

    Ok(())
}

async fn my_handler(
    e: rusoto_dynamodbstreams::GetRecordsOutput,
    _c: Context,
) -> Result<CustomOutput, HandlerError> {
    use futures::stream::StreamExt;
    futures::stream::iter(e.records.unwrap()).for_each_concurrent(1, async move |record| {
        println!("{:?}", record);
        match record.event_name.as_ref().map(String::as_str) {
            Some("REMOVE") | None => {}
            Some("MODIFY") | Some("INSERT") => {
                let record_data = YelpBusiness::from_attrs(
                    record
                        .dynamodb
                        .unwrap()
                        .new_image
                        .unwrap()
                        .into_iter()
                        .map(|(k, v)| (k, to_dynomite(v)))
                        .collect(),
                )
                .unwrap();
                let dynamo_id = record_data.id.to_owned();
                let cuisines: Vec<_> = record_data.categories.iter().map(|category|
                                                                         category.title.to_owned()
                ).collect();
                let item =                     YelpBusinessEs {
                        id:  dynamo_id.to_owned(),
                        dynamo_id: dynamo_id.to_owned(),
                        cuisines: cuisines
                    };
                        let creds = rusoto_credential::DefaultCredentialsProvider::new()
                            .unwrap()
                            .credentials()
                            .await
                            .unwrap();
                        let serialized = serde_json::to_string(&item).unwrap();
                        let mut request = rusoto_core::signature::SignedRequest::new(
                            "PUT",
                            "es",
                            &Region::UsEast1,
                            &format!("/yelp_restaurants/_doc/{}", &item.id),
                        );
                        request.set_content_type("application/json".to_owned());
                        request.set_hostname(Some("vpc-yelp-restaurants-e5ebs6h5mdt6pbvf3lcphwpe5u.us-east-1.es.amazonaws.com".to_owned()));
                        request.set_payload(Some(serialized));
                        request.set_content_md5_header();
                        request.sign(&creds);
                        let client = rusoto_core::request::HttpClient::new().unwrap();
                        let mut response = client.dispatch(request, None).await.unwrap();
                        let buffer = response.buffer().await.unwrap();
                        let response = buffer.body_as_str();
                        println!("{}", response); 
            }
            Some(weird) => println!("Unexpected: {}", weird),
        }
    }).await;
    Ok(CustomOutput {})
}
fn my_outer_handler(
    e: rusoto_dynamodbstreams::GetRecordsOutput,
    c: Context,
) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e, c))
}
