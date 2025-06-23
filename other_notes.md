### Contitional code for
## Cargo.toml ...
[target.'cfg(target_arch = "wasm32")'.features]
[target.'cfg(target_arch = "wasm32")'.dev-dependencies]
[target.'cfg(target_arch = "aarch64")'.dev-dependencies]
## and Rust.rs:
 #[cfg(not(target_arch = "wasm32"))]


## Examples and Test Ways:
let mut id = 45696973; //369161987; // 121486088 369161987; //159243621; // 437150850 //  121486088
let show_only: u64 = 0; // 159243622; //1174629866; //1174306433;

* Oriental Pearl Tower (40778038) Relation: Saint Basil's Cathedral (3030568)
Testing with a moderate complex building OR a lage complex one
* https://www.openstreetmap.org/way/121486088

* taj_mahal_id = 375257537;
* marienplatz_id = 223907278; // 15
* fo_gabled = 45696973; // rectangle: 47942624 +schräg: 45697283  haustür: 47942638  eingeeckt: 45697162  winklel: 45402141
* no roof 45697280 BADs!: 45697037, 45402130  +OK+: 37616289
* Not valide tagged???: 45696973 bowed building, no part
* dome: part: 159243621   buiding: 437150850

* Reifenberg  121486088
* Passau Dom 24771505    gabled: 464090146   unten: 136144290  oben: 136144289
* St Paul's Cathedral 369161987  -  Test center gabled: 664642004 (footprint is rounded!)
* Westminster  367642719 / Abbey 364313092
* Vatikan, Saint Peter's Basilica (244159210)
* Cathedral of Notre Dame (201611261)
* Hagia Sophia (Holy Wisdom) (109862851)

### Notes on WASM:
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
