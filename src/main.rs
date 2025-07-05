// This file is a copy from the example m_async.rs
// It is needed to debug with Zed
// use tokio; was added AND! Cargo.toml was changed! tokio now is NOT only for aarch64

use osm_tb::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // you could move the code of the function main_async out of the lib and insert here, to modify it as you like.
    main_async().await?;
    Ok(())
}
