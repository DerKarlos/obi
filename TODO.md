Ideas from https://github.com/Beakerboy/OSMBuilding/tree/main
* errorBox: für ipad / noch mal das text ui probieren! Example
* surrounds(shape, point) & partIsInside für Part in Building! Interessant!
     Nicht langsam, aber auch kein grobcheck

Next todo:
* crash: way 1149973649
* Relation with > 1 outer and >1 for one inner: 8035487.    9346128=4outer
* Contributing hat gute punkte und meine hilfsbedürfte dazu
* And also crashes: 3376015
* Accept relations (auto if nuber < 1000xxx?)
* Overtures als datenquelle:
  https://docs.overturemaps.org/guides/buildings/#14/32.58453/-117.05154/0/60
  https://github.com/andersborgabiro/overture2stl
* one line for   ExtrudeRing { ...  macro?!
* building:part inherits every building* tags from their parent
* Big Ben clock half missing: roof:shape=round
* Gute Tests, wenigstens absturztests!
* Not simplyfy but delete if >60% covered by parts. How? Triangluate? is area() in lib?
  Example: Remaining of rel: cargo run --example m_async -- -w 111355120 -o 1567133
      use i_overlay::core::overlay::{ContourDirection, IntOverlayOptions, Overlay};
      use i_overlay::core::solver::Solver;
          let options = IntOverlayOptions {
            preserve_input_collinear: false,
            output_direction: ContourDirection::Clockwise,
            preserve_output_collinear: false,
            min_output_area: 1234,
        };
        //Overlay::with_contours_custom(&test.subj_paths, &test.clip_paths, options, solver);
        Overlay::with_contours_custom(&self.polygons, &test.clip_paths, options, Solver::AUTO);

        https://www.omnicalculator.com/math/triangle-area
        area = 0.25 × √( (a + b + c) × (-a + b + c) × (a - b + c) × (a + b - c) )


More todo:
* Solve this DOUBLE outer https://www.openstreetmap.org/relation/14548261#map=19/51.517498/-0.102085
* Historymap ist nur andere url
* https://wiki.f4map.com/render
* Kölner Dom: part 1233649406 has only building:material=stone, but F4 shows the same color as
   691226039 with no color or material. Does it get it from the relation?
   relation 4532022 has building:colour=#726555, building:material=stone ?
   https://demo.f4map.com/#lat=50.9413230&lon=6.9581705&zoom=20&camera.theta=58.801&camera.phi=-6.016
   https://www.openstreetmap.org/#map=19/50.941272/6.958028
* Kölner Dom: with range>=271, there is a strange L-shape-spike
  https://osmgo.org/bevy.html?way=1233649406&range=271&only=4532022

* Make tests. See https://github.com/expobrain/openstreetmap-api-rs/tree/master/tests
* When needs a buidling als to be a part? way 1149973649 is just a building. Overpass vor both and check
* Use tag!: roof:angle=10 (degrees)
* Use for roof cut: https://github.com/iShape-Rust/iOverlay/tree/main/iOverlay
* Build of rend3 is so fast. Incremental build? Use it in OBI/OTB!
* Use OMA Files: https://community.openstreetmap.org/t/a-rust-lib-crate-to-read-oma-files/129593
* Flickering and no roof in OSMgo: Way 1031078075
* Some GPU-Warings from Bevy in the JS console
* Check WebGL and WebGPU: https://bevyengine.org/examples/3d-rendering/3d-scene/
* -1 Tagging: https://www.openstreetmap.org/node/12231809761 (see mails/messages, -1 is 0-1 layers for post codes. Building in Building?)
* Panoramax: ask for an API "nearest" or use bbox and sort in the code
* subtract Test3: 278033615 1125067806 todo: part is > building! Subtraktion deletes level 0
* A strange part at the Gib Ben west side goes up skillion above the tower
* https://www.openstreetmap.org/way/172649356 should be subtracted to 0 but an infinite line remains. if area < 0.1 m*m: eliminate
* building part 367642675 with roof:height looks wrong and is not 24 as the next part
* how to handle roof:levels = > 0 and no roof:type ?  way 138462520

* Is the crate https://crates.io/crates/osm-api able to build wasm and deliver the needed data? If yes, a new input "modul" is needed. A good way to fine the division of the "OSM-Toolbox"

* edition = "2024" causes error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
* Show crash stack. Dose not work:  std::env::set_var("RUST_LIB_BACKTRACE", "1");

* DONT USE?:  https://api.openstreetmap.org/api/0.6/way/121486088/full.json
  https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
  The test-server does not have needed objects (like Reifenberg), but they could be PUT into

* bevy_web_asset = { version = "0.11.0", optional = true } # "This is a tiny crate that that adds the ability to load assets from http and https urls. Supports both wasm (web-sys) and native."
  do?: Using fetch browser API • https://rustwasm.github.io/wasm-bindgen/examples/fetch.html
  web-sys, js-sys, wasm-bindgen and wasm-bindgen-futures • https://crates.io/crates/ehttp

F4map
* building:colour = roof:colour? part: 207377042?
  But 206020152 below with roof:material=stone looks the same
* Thin roofs-trick:
  https://demo.f4map.com/#lat=50.1694588&lon=8.6372565&zoom=18&camera.theta=58.514&camera.phi=8.021&lat=50.1688086&lon=8.6388286
  S-GL: https://streets.gl/#50.1688086,8.6388286,45.00,0.00,2000.00
  https://osmgo.org/go.html?lat=50.1688086&lon=8.6388286238025&dir=0&view=-45&ele=555

The API:Lib  Info: XML doesn't have arrays, just multible lines with the same "ids" for node or tag
* Contakt the creator (by an issue in repo) Tell what you did and what deltas (element sort) to json and problems and wishes (HashMap)
