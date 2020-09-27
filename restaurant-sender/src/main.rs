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
use serde::{Serialize, Deserialize};
use shared_types::{YelpBusiness, YelpBusinessEs};
use simple_logger;
use std::error::Error;
use rusoto_sns::{Sns, SnsClient, PublishInput};
use rusoto_core::{Region};

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::max());
    lambda!(my_handler_outer);

    Ok(())
}

async fn my_handler(
    e: RestaurantRequest,
) -> Result<CustomOutput, HandlerError> {
    Ok(CustomOutput{
        phonenumber: e.phonenumber,
        message: "Hi".to_owned(),
    })
}
fn my_handler_outer(
    e: RestaurantRequest,
    _c: Context,
) -> Result<CustomOutput, HandlerError> {
    futures::executor::block_on(my_handler(e))
}
