* Use OMA Files: https://community.openstreetmap.org/t/a-rust-lib-crate-to-read-oma-files/129593
* Flickering and no roof in OSMgo: Way 1031078075
* Some GPU-Warings from Bevy in the JS console
* Check WebGL and WebGPU: https://bevyengine.org/examples/3d-rendering/3d-scene/

* Is the crate https://crates.io/crates/osm-api able to build wasm and deliver the needed data? If yes, a new input "modul" is needed. A good way to fine the division of the "OSM-Toolbox"

* edition = "2024" causes error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
* Show crash stack. Dose not work:  std::env::set_var("RUST_LIB_BACKTRACE", "1");

* DONT USE?:  https://api.openstreetmap.org/api/0.6/way/121486088/full.json
  https://master.apis.dev.openstreetmap.org/api/0.6/way/121486088/full.json
  The test-server does not have needed objects (like Reifenberg), but they could be PUT into

F4map thin roofs-trick:
* https://demo.f4map.com/#lat=50.1694588&lon=8.6372565&zoom=18&camera.theta=58.514&camera.phi=8.021&lat=50.1688086&lon=8.6388286
* S-GL: https://streets.gl/#50.1688086,8.6388286,45.00,0.00,2000.00
* https://osmgo.org/go.html?lat=50.1688086&lon=8.6388286238025&dir=0&view=-45&ele=555
