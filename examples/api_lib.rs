// Is the crate https://crates.io/crates/osm-api able to build wasm and deliver the needed data?
// If yes, a new input "modul" is needed. A good way to fine the division of the "OSM-Toolbox"

use openstreetmap_api::Openstreetmap;
use openstreetmap_api::types::Credentials;
//use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // It was not easy to ques, what realy is needed:
    let host = "https://api.openstreetmap.org/api".to_string(); // env::var("OPENSTREETMAP_HOST")?;
    //let user = "".to_string(); // env::var("OPENSTREETMAP_USER")?;
    //let password = "".to_string(); // env::var("OPENSTREETMAP_PASSWORD")?;
    let credentials = Credentials::None;
    let client = Openstreetmap::new(host, credentials);

    let v = client.versions().await?;
    println!("{:?}", v);

    let w = client.ways().get(121486088).await?;
    println!("{:?}", w);

    Ok(())
}
