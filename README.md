# "OBI" - OSM Building Inspector (working title)

Decades ago, www.OSMgo.org was started and went from a game to a "full" 3D viewer of OSM.
As one asked vor materials support, this redoing of OSMgo in Rust was born.
But only for single buildings and building-parts for now.
And it may get editor features someday. (renamed to OBE?)

### Why Rust?
Why not? It is hard to learn! I would use Zig, if someone taking response for Zig use joins this project.
Why? Because I just like it. Because it builds to any target: Desktop, Mobile, Web. Because it has a unique save memory management.
There are some others coding Rust for OSM: https://en.osm.town/@amapanda and the one, doing MapLibre.RS (Todo)
More: https://mary.codes/blog/programming/generating_a_vr_map_with_osm_and_aframe/

### Why Bevy?
Yes, there are less complicated 3D renderer. And all of them use WGPU anyway. May be just WGPU would do it? Join this project and do it.
The 3D-rendering-part of this project may become a separate crate and a switch may select different renderer.
Just in case, this Building viewer/editor may become more, Bevy would allow gamification.

### Positions concept
Using Bevy, a 2D game would use X and Y for a map. But as this project is more a 3D game, the game level floor is x and -z. Minus because a positife longitude is shown in the backround, which is -z in the Bevy GPU.

Used structure types:
* GeoPos: lat and lon, in f64 to get acurate meters while subtracting the GPU zero position from the actual node positions.
* XzPos: x and z in 32, as needed for the GPU. Z already inverted.

* OsmNode: A XzPos and optional tagging.

* JosnData: Main struct of the OSM API return, with all Elements
* JsonElement: May be a Node, Way or a Relation. May have a geo-position, may have tagging
* JosnTags: TODO: should be a Map:  https://serde.rs/deserialize-map.html
