use osm_tb::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // you could move the code of the function obi_async out of the lib and insert here, to modify it as you like.
    obi_async().await?;
    Ok(())
}
