# "OBI" or "OSM-BI" - OSM Building Inspector

This tool displays a single OSM building rendered in 3D, to inspect whether the edited OSM tags show the expected view. It was inspired by [Beakerboy's OSMBuilding](https://github.com/Beakerboy/OSMBuilding) and uses some know-how of [www.OSMgo.org](https://www.osmgo.org) and [www.OSM2World.org](https://www.OSM2World.org). It may get editor features and more some day.

It renders buildings and its building-parts with some of the roof:types. It shows colours and colours of material types.

The control supports keys, mouse and touch, like F4map but extendet:
* Arrow keys: rotate left/right and move forward/backward
* ADSW: move TG up down , rotate: QE left/right, RF up/down Y=Z/H zoom
* Number Key 0:  Reset the building position.
* Mouse keys first/2nd: rotate (move in tile mode) Mouse wheel: zoom
* Single/Double Touch: rotate (move in tile mode) pinch: zoom
* Tribble Touch: move up/down and right/left

The building must be given as a OSM way or relation ID:
* ```http://www.OSMgo.org/obi?way=24771505```
* ```cargo run --example wasm -- --way 24771505```

Try it:
[default: St.Pauls](http://www.OSMgo.org/bevy.html),
[Westminster](https://www.osmgo.org/bevy.html?rel=1567699&aray=140)
-[Aby](https://www.osmgo.org/bevy.html?way=364313092),
[London-City](https://osmgo.org/bevy.html?rel=3850933&area=444),
[Hagia Sophia](https://osmgo.org/bevy.html?way=109862851),
[Krahnhäuser](https://osmgo.org/bevy.html?rel=184729&area=333)
* BakerBoy's examples:
[Ryugyong Hotel ](https://osmgo.org/bevy.html?way=407995274),
[The Bandstand](https://osmgo.org/bevy.html?way=297997038)
[Nidaros Cathedral](https://osmgo.org/bevy.html?way=417245741)
[Castle of the Holy Angel](https://osmgo.org/bevy.html?way=8035487), [](), [](), [](), [](), []()

To get updates for this project, you may follow [this Fediverse account](https://en.osm.town/@rust_osm_tb)

In this repository there is a lib to
- read OSM data,
- sort the tagging into "layers" with the needed values,
- calculate all 3D triangles
- and render them by an engine (Bevy and Rend3 at the moment).

There are two ways to read the OSM API. See the rust example "async" how to use input-handler alternatively, by just changing the name of the handler:
- A rust source to directly read the API as Json, or
- A code using the crate ([openstreetmap-api](https://github.com/topics/openstreetmap-api)) which uses the XML API.

The Json handler supports async and blocked acces to the OSM API. But there seems no way to build to WASM for a Web-App, neither with blocked nor with async. Fortunately there is a Rust example for Bevy, using the Bevy asset loader. Magically this runs in WASM and si the web-app, descripted abowe.

The exampe for Rend3 is still in development. There is still hope, it will also run on the web. May be, it will take les download time, as I hope.

### Why Rust?
Why not? Yes, it is hard to learn firstly. I considered to use Zig but can't go back to manual memory management after TypeScript and Rust. If someone will take response for Zig usage, I would joins this project to. Rust because it builds to any target: Desktop, mobile, Web. And I like the habit of Rust to write creates/libs to be reused. There are quite some crates for OSM. And projects, too:
The [WaterMap]( https://en.osm.town/@amapanda),
[MapLibre.RS](https://github.com/maplibre/maplibre-rs)

### Zed and Codeberg
I wanted to use the new Zed IDE instead of VS-Code, because it is not from MicroSoft, written in Rust, it offers AI functions built in. Works great so far, but there is no Debugging yet.
I wanted to use CodeBerg (or GitLab or SourceHud) as Repository website to get away from MicroSoft and to Europe may be. But I could not get it running yet, so it is still on GitHub.

### Why Bevy?
Less complicated 3D render engines are added too. And all of them use WGPU anyway. Just WGPU would do it? If you like, join this project and try it. The 3D-rendering-part of this project may become a separate crate or build features may select different renderer.
Just in case, this Building viewer/editor may become more, Bevy would allow for gamification.

### Geo- and GPU Positions concept
Using Bevy, a 2D game would use X and Y for a map. But as this project is more a 3D game, the game level floor is x and -z. Minus because a positive longitude is shown in the backround, which is -z in the Bevy GPU. East is +latitude is +x; north is +longitude, but is -z! THis definition is capsuled in a function and may be changed to other coordinate systems. Or they may get dynamic selectable.

### Data sources
For now it directly uses the OSM API for editors. Overpass-Turbo is possible, but may be slightly slower in updating edited tags.
Even Vector tiles may become an alternative, OSM or Overture, if they support building:parts one day. I don't expect that in the deault tiles but in an optional tile-file which includes all(!) the details, not needed for "normal" 2D map rendering.

### Project patterns
* As used to be in Rust, don't use abbreviations. Exceptions should be documented
* Always latitude before longitude and north before east, in the code
* Now and then check for all clone() and copy() to be really needed. And for Todo, ttt and ??? markers in the source codes

### Lib-Structure
* Existing input modules are: input_osm_json.rs and input_osm_lib.rs. They keep the received data structure internal.
* The OSM Tagging modules: osm2layers.rs and shape.rs are called by the by the input modules.
* The 3D shape of the building is generated by render_3d.rs. Later, there may also be 2D renderers.
* The visualisation by the GPU is done by bevy_ui.rs now. Other engines and a GLB-file output is intendet to.

### Used structure types
* GeographicCoordinates: latitude and longitude, in f64 to get accurate meters while subtracting the GPU zero position from the actual node position.
* GroundPosition: north and east in 32, as needed for the GPU.
* BoundingBox: north south east and west values. May be used for GPU or geographic values.
* BuildingOrPart: is the interface from the input modules to the 3D generation
* OsmMeshAttributes: is the interface from the 3D generation to the render engine
* JosnData: Main struct of the OSM API return, with all Elements
* JsonElement: May be a Node, Way or a Relation. May have a geo-position, may have tagging
* JosnTags: TODO: should be a Map:  https://serde.rs/deserialize-map.html

### Trouble maker?
* Never ending discussion about 100% part coverage: https://github.com/StrandedKitty/streets-gl/issues/3

<!-- https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax -->
