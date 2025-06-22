use osm_tb::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // you could move the code of the function main_async out of the lib and insert here, to modify it as you like.
    main_async().await?;
    Ok(())
}
