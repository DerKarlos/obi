
use error_chain::error_chain;
//use std::io::Read;
use std::fmt;
use reqwest;
use serde::Deserialize;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[derive(Deserialize, Debug)]
struct Element {
    #[serde(rename = "type")]
    element_type: String,
}

#[derive(Deserialize, Debug)]
struct Json {
    version: String,
    generator: String,
    elements: Vec<Element>,
}

// https://stackoverflow.com/questions/54488320/how-to-implement-display-on-a-trait-object-where-the-types-already-implement-dis
impl fmt::Display for Json{

    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
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

    let json: Json = reqwest::blocking::get(url)?.json()?;

    //let res    = reqwest::blocking::get(url)?;
    //let json: Json = res.json()?;
    //println!("Get status: {}\n", res.status());
    //println!("Headers:\n{:#?}", res.headers());

    //let mut body = String::new();
    //res.read_to_string(&mut body)?;
    //println!("Body:\n{}", body);
    //let json: Json = serde_json::from_str(&body).unwrap();

    //println!("json = {:?}", json);
    println!("version = {:?}", json.version.parse::<f32>().unwrap() );
    println!("generator = {:?}", json.generator);
    //println!("elements = {:?}", deserialized.elements);
    for element in json.elements {
        //println!("element = {:?}", element);
        println!("type = {:?}", element.element_type);
    }

    Ok(())

}
