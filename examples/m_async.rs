use osm_tb::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    main_async().await?;
    Ok(())
}
