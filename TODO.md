To be discussed:
* Hilton Building: 107643543  65% outer left and  F4 renders the remaining
* Part 440123085 4 levels AND withRoof:levels=1!  The F4map roof is not exact the L shape (also building 440257472)

Next todo:
* wasm auch mit Meldungen wie Ist Part/Relation etc.
* part 1144964446 gone by other ???
* Select part for OSM ID page view
* Missing a lot: way 228691410 ( Saint Basil's Cathedral Relation 3030568)
* München Rathaus Erker-parts outside building. https://www.openstreetmap.org/way/224253365

* errorBox/ui: für ipad
* A paramter rel=9593239 sould help with the outer
* Flickerings around St Pauls, and elsewere
* Missing parts near 109862851 (other moskue)
* An odd roof near 417245741
* If one inspects an existing building, edited for the F4map renderer, it will look "wrong".
   (I need to analyse, how F4Map logic is here)
* part 1373331436 not inside building 172649356 !?
* crash: way 1149973649
* Relation with > 1 outer and >1 for one inner: 8035487.    9346128=4outer
* https://www.openstreetmap.org/way/229711939   odd roof: direction "E" add snapping
* Contributing hat gute punkte und meine hilfsbedürfte dazu
* And also crashes: 3376015
* Accept relations (auto if nuber < 1000xxx?)
* 440100148 is NOT inside 107643502 as other_is_inside tells! Gets not subtracted from (107643498) !!!
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
* LEFT part: 320420934 is mostly outside the building (no relation) AND UNDERGROUND
* 316556318: Dach-rel:4166727 geht one -r 1 nicht? The inverted Bakerboy-fn gives a node outside!
* building:part=steps - https://www.openstreetmap.org/way/311294175#map=18/52.239348/21.046624
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

* Get this warings away:
  GPU preprocessing is not supported on this device. Falling back to CPU preprocessing.
  ScreenSpaceAmbientOcclusionPlugin not loaded. GPU lacks support: TextureFormat::R16Float does not support TextureUsages::STORAGE_BINDING.
  AtmospherePlugin not loaded. GPU lacks support for compute shaders.

F4map

* https://wiki.f4map.com/render
* building:colour = roof:colour? part: 207377042?
  But 206020152 below with roof:material=stone looks the same
* Thin roofs-trick:
  https://demo.f4map.com/#lat=50.1694588&lon=8.6372565&zoom=18&camera.theta=58.514&camera.phi=8.021&lat=50.1688086&lon=8.6388286
  S-GL: https://streets.gl/#50.1688086,8.6388286,45.00,0.00,2000.00
  https://osmgo.org/go.html?lat=50.1688086&lon=8.6388286238025&dir=0&view=-45&ele=555

The API:Lib  Info: XML doesn't have arrays, just multible lines with the same "ids" for node or tag
* Contakt the creator (by an issue in repo) Tell what you did and what deltas (element sort) to json and problems and wishes (HashMap)
