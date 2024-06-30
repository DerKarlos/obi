
use error_chain::error_chain;
use std::io::Read;
use reqwest;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

fn main() -> Result<()> {    
    //env_logger::init();    //log::debug!("xxx: {}", xxx);
 
    println!("Hi, I'm OBI, the OSM Buiding Inspector");
    // Reifenberg Kirche Way 121486088
    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    // window.open(httpx+"www.openstreetmap.org/way/121486088"_blank")
    // -           https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json does not have that way, 12148 works.
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = "https://api.openstreetmap.org/api/0.6/way/121486088/full.json";

    // https://rust-lang-nursery.github.io/rust-cookbook/web/clients/requests.html
    // https://docs-rs-web-prod.infra.rust-lang.org/reqwest/0.10.0/reqwest/blocking/index.html

    let mut res = reqwest::blocking::get(url)?;
    let mut body = String::new();
    res.read_to_string(&mut body)?;

    println!("Status: {}", res.status());
    //intln!("Headers:\n{:#?}", res.headers());
    println!("Body:\n{}", body);

    Ok(())

}
