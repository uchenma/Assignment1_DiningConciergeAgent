use std::error::Error;

use dynomite::{
    Attributes, FromAttributes, Item,
};
use elasticsearch::{auth::Credentials,http::{Url,transport::{SingleNodeConnectionPool,TransportBuilder}}, Elasticsearch, IndexParts};
use lambda_runtime::{error::HandlerError, lambda, Context};
use log::{self};
use serde::{Deserialize, Serialize};
use simple_logger;

#[derive(Attributes, Deserialize, Serialize, Debug, Clone)]
struct YelpCategory {
    alias: String,
    title: String,
}
#[derive(Item, Deserialize, Serialize, Debug, Clone)]
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
    lambda!(my_handler);

    Ok(())
}


fn my_handler(
    e: rusoto_dynamodbstreams::GetRecordsOutput,
    _c: Context,
) -> Result<CustomOutput, HandlerError> {
    let es_user = std::env::var("ES_USER").unwrap();
    let es_pass = std::env::var("ES_PASS").unwrap();
    let transport = TransportBuilder::new(SingleNodeConnectionPool::new(
        Url::parse("https://vpc-yelp-restaurants-afhintr5ppa3f4vhraxvlhmvti.us-east-2.es.amazonaws.com").unwrap()
    ))
        .auth(Credentials::Basic(es_user, es_pass))
    .build()
    .unwrap();
    let client = Elasticsearch::new(transport);
    e.records.unwrap().into_iter().for_each(|record| {
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
                let serialized = serde_json::to_value(&record_data).unwrap();
                futures::executor::block_on(
                    client
                        .index(IndexParts::IndexId("yelp_restaurants", &record_data.id))
                        .body(serialized)
                        .send(),
                )
                .unwrap();
            }
            Some(weird) => println!("Unexpected: {}", weird),
        }
    });
    Ok(CustomOutput {})
}
