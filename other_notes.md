### Contitional code for
## Cargo.toml ...
[target.'cfg(target_arch = "wasm32")'.features]
[target.'cfg(target_arch = "wasm32")'.dev-dependencies]
[target.'cfg(target_arch = "aarch64")'.dev-dependencies]
## and Rust.rs:
 #[cfg(not(target_arch = "wasm32"))]


## Examples and Test Ways:

Most used tests
* Reifenberg  121486088
* Relation Bau 46: r 2819147 w 45590896
* Passau Dom 24771505 gabled: 464090146 skilleon south: 136144290  north: 136144289
   todo: split https://www.openstreetmap.org/way/464090137 in gabled and phyramide
* St Paul's Cathedral 369161987 center gabled: 664642004
   todo: spilt https://www.openstreetmap.org/way/664642004 in gabled and phyramide
* Westminster  r 1567699, w 367642719  w633493242+range1! / Abbey 364313092

* gabled: rectangle/square: 47942624 rotated: 45697283 door: 47942638 corneered: 45697162
          angled: 45402141 is correkt but we expect and agle gabled, hm?
          komplex: 37616289, 45697037 ok with "across-backup-code": 45402130  bad_bow:45696973 do parts!

More Examples
* domes.. round_dome:437150850, outer:159243621=building:437150850+square_dome:159243622,
* Saint Basil's Cathedral Relation 3030568 - many small outers!  dont: w-228691410
* Vatikan, Saint Peter's Basilica  244159210
* Marienplatz Rathaus: p 147095 w-23632633 part: 223907278
* Cologne Cathedral: r 2788226 w 1233649406 outer-way 4532022,     Krahnhaus: r 184729, w 234160726
* https://osmgo.org/bevy.html?way=398036478 one more Ernest Pohl Stadium  Polen

Subtract tests
* Test1: cargo run --example m_async -- -r 0 -w 239592652
* This way does NOT set the height/levels to maximum
* Test2: building: 278033600 + two parts: 1124726437 1124499584
* Test3: 278033615 1125067806 todo: part is > building! Subtraktion deletes level 0
* Test4: rel 2111466 Outer=building 157880278 Parts 109458125 1104998081 1104998082
* Test5: way 111354081 parts: 814784273 + 814784274 + 814784275
* Test6: rel 11192041: outer 172664019, inner 814784298
* Test7: building: 440100162 - parts: 107643530, 440100141 = empty  Todo: tunnel=building_passage

seule
https://beakerboy.github.io/OSMBuilding/index.html?type=relation&id=3376015
https://osmgo.org/bevy.html?way=251621058
burg
https://beakerboy.github.io/OSMBuilding/index.html?id=8035487
https://osmgo.org/bevy.html?way=8035487
Moskau
http://osmgo.org/bevy.html?way=228697049&range=100

### Notes on WASM:  run server localhost
https://www.youtube.com/watch?v=VjXiREbPtJs % rustup update
% rustup target add wasm32-unknown-unknown
% cargo install -f wasm-bindgen-cli    !!! AT THE PROJECT DIR !!!
% cargo build --release --target wasm32-unknown-unknown
% wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release(/examples)/<name>.wasm
% npx serve .

cargo build --example bevy --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/examples/bevy.wasm

“blocking” was not supported for WASM. Change (Bevy-)Code code do NON-blocking
https://lib.rs/crates/bevy-async-runner
https://docs.rs/crate/bevy-tokio-tasks/latest/source/examples/current_thread_runtime.rs

https://dev.to/sbelzile/making-games-in-rust-deploying-a-bevy-app-to-the-web-1ahn

EURORUST @euro_rust Building BEVY, WASM AND THE BROWSER FRANÇOIS MOCKERS OPTIMISING
• For size • Smaller file to download • Less code to run • Disable Bevy features you don't need
• The best code to optimise is the one not even compiled • A step further
• https://github.com/WebAssembly/binaryen
, https://rustwasm.github.io/docs/book/reference/code-size.html
• https://github.com/rustwasm/twiggy
• https://github.com/rustwasm/wasm-snip

### Notes on Zed:
- The term and handling of “project” is not descriptor. I found only a “Find in Project”
- Overtype / Exchange Mode, not Insert: Zig default is like VScode: no overtype, but an extension NOT vor Zed

Key-bindings: cmd-shift-r Select and run task. cmd-opt-r Run last task again. Opt-cmd-down Block select.


### All settings of BEVY
"android-game-activity",
"android_shared_stdcxx",
"animation",
"async_executor",
"bevy_asset",
"bevy_audio",
"bevy_color",
"bevy_core_pipeline",
"bevy_gilrs",
"bevy_gizmos",
"bevy_gltf",
"bevy_input_focus",
"bevy_log",
"bevy_mesh_picking_backend",
"bevy_pbr",
"bevy_picking",
"bevy_render",
"bevy_scene",
"bevy_sprite",
"bevy_sprite_picking_backend",
"bevy_state",
"bevy_text",
"bevy_ui",
"bevy_ui_picking_backend",
"bevy_window",
"bevy_winit",
"custom_cursor",
"default_font",
"hdr",
"multi_threaded",
"png",
"smaa_luts",
"std",
"sysinfo_plugin",
"tonemapping_luts",
"vorbis",
"webgl2",
"x11",




## Notes about the history of OBI/OTB:

As mentiond in my [OSM daiary](https://www.openstreetmap.org/user/-karlos-/diary/406592), The 3D Tool [*OSMBuilding*](https://github.com/Beakerboy/OSMBuilding) from Beakerboy not only motivated me to add roof:type "gabled" in [OSMgo](https://www.osmgo.org). It also inspired me, to continue recoding some "OSM 3D rendering" in **Rust**


The algorithm of *OSMBuilding* for gabled had some clitches. And I did not understand it. As Zed offered AI, I asked for an algorithm. That clever AI did see the existing Javascript code and transcoded it to Rust; with some erros, but it worked at last the same way. So I ordered the AI to write a plain new algorithm. Yes, this was understandable. It will not work for buildings with holes. Later ...

Decades ago, [www.OSMgo.org](https://www.osmgo.org) was started and went from a "game" to a "symbolic" 3D viewer of OSM.
As one asked vor materials support, this redoing of OSMgo in **Rust** was started,

 using a new IDE **Zed**. Firstly only alike *OSMBuilding*, later more.



## Notes about wrong tagging. Changing it in the OSM database?
* Rel 6137280 has a member  410265429 as type outline instead of outer  How to use Overpass?


Notes on Bakerboy code:
* isSelfIntersecting geht nur, wenn node an Kreuzung ist
* Center is bbox-center
* getWidth ist getMaxExtend
* vertexAngle wozu? Nur für test
* Test the three lights ... no: Ambient does not exist, 1 blocks control
