
use error_chain::error_chain;
use reqwest;
use serde::Deserialize;
use std::collections::HashMap;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[derive(Deserialize, Debug)]
struct JosnElement {
    #[serde(rename = "type")]
    element_type: String,
    id: u64,
    lat: Option<f64>,
    lon: Option<f64>,
    nodes: Option<Vec<u64>>,
    tags: Option<Tags>,
}

#[derive(Deserialize, Debug)]
struct Tags {
    building: Option<String>,
    name: Option<String>,
}



#[derive(Deserialize, Debug)]
struct JsonData {
    elements: Vec<JosnElement>,
}


struct OsmNode {
    lat: f64,
    lon: f64,
}

fn main() -> Result<()> {    
 
    println!("Hi, I'm OBI, the OSM Buiding Inspector");

    // Reifenberg Kirche Way 121486088
    const WAY_ID: u64 = 121486088;

    // https://www.openstreetmap.org/way/121486088#map=19/49.75594/11.13575&layers=D
    // window.open(httpx+"www.openstreetmap.org/way/121486088"_blank")
    // -           https://api.openstreetmap.org/api/0.6/way/121486088/full.json
    // https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json does not have that way, 12148 works.
    // The test-server does not have needed objects (like Reifenberg), but they could be PUT into

    let url = format!("https://api.openstreetmap.org/api/0.6/way/{}/full.json", WAY_ID);

    let json: JsonData = reqwest::blocking::get(url)?.json()?;

    //let nodes = Map(id:u64, node:OsmNode);
    let mut nodes_map = HashMap::new();

    for element in json.elements {
        if element.element_type == "node".to_string() {
            let osm_node = OsmNode{lat: element.lat.unwrap(), lon: element.lon.unwrap(),};
            nodes_map.insert(element.id, osm_node);
            // println!("Node: id = {:?} lat = {:?} lon = {:?}", element.id, element.lat.unwrap(), element.lon.unwrap() );
        }
        if element.element_type == "way".to_string() {
            let id = element.id;
            let nodes = element.nodes.unwrap();
            let tags = element.tags.unwrap();
            let name = tags.name.unwrap();
            let building = tags.building.unwrap();

            println!(" Way: id = {:?}  building = {:?}  name = {:?}",
                id,
                building,
                name,
            );

            let mut lat_min: f64 = 1e9;
            let mut lon_min: f64 = 1e9;
            let mut lat_max: f64 = -1e9;
            let mut lon_max: f64 = -1e9;

            for node_id in nodes {
                let node = nodes_map.get(&node_id).unwrap();
                lat_min = lat_min.min(node.lat);
                lat_max = lat_max.max(node.lat);
                lon_min = lon_min.min(node.lon);
                lon_max = lon_max.max(node.lon);
                println!("Way-Node: id = {:?} lat = {:?} lon = {:?}", node_id, node.lat, node.lon );
            }
            let center_lat = lat_min + (lat_max-lat_min)/2.0;
            let center_lon = lon_min + (lon_max-lon_min)/2.0;
            println!("Way-center: lat = {:?} lon = {:?}", center_lat, center_lon );

        }
    }

    Ok(())

}
