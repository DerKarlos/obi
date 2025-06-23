* https://wiki.openstreetmap.org/wiki/Simple_3D_Buildings

* Find interesting buildings with Overpass: https://overpass-turbo.eu/?template=key-value&key=roof%3Ashape&value=round

* Find Suspicious OpenStreetMap Changeset:
https://resultmaps.neis-one.org/osm-suspicious?country=184&hours=48&tsearch=hydrant&mappingdays=3&anyobj=t&comp=%3E&value=10&action=d&obj=n&filterkey=#12/51.1576/6.7171

* To search "wrong" colour (much Output; how to only ID/colour-value?):  [out:json][timeout:25]; nwr["building:part"]["colour"]({{bbox}}); out geom;


* For an Overlay/Filter?: https://github.com/openinframap/josm-power-network-tools/

* Indoor 2025: https://volkerkrause.eu/2025/04/05/osm-fossgis-conference-2025.html
* Indoor Data import: https://community.openstreetmap.org/t/proposed-import-of-the-indoor-data-offered-for-osm-by-the-technical-university-of-munich-tum/127003




### Beakerboy's OSMBuilding:
Git:  https://github.com/Beakerboy/OSMBuilding
and  https://github.com/Beakerboy/Threejs-Geometries
Test: https://beakerboy.github.io/OSMBuilding/index.html?id=1567699&type=rel


### OSM crates in Rust:
https://wiki.openstreetmap.org/wiki/Rust
https://crates.io/keywords/osm or openstreetmap
https://crates.io/search?q=osm or openstreetmap
#### OSM-API:
https://crates.io/crates/osm-api
https://crates.io/crates/openstreetmap-api
https://crates.io/crates/osm-cli
https://crates.io/crates/geogetter
#### Other Crates:
Rust generates A-Frame: More: https://mary.codes/blog/programming/generating_a_vr_map_with_osm_and_aframe/

### Videos:
* 2025 Tobias Kerr: https://media.ccc.de/v/fossgis2025-57391-osm2world-updates-fur-den-3d-pionier
* Building a game engine in Rust https://fyrox.rs https://youtu.be/ao4mTUgZ4H4?si=aUpI8AEMneyE8eny


### Projects may be interresting:
* https://www.geofabrik.de/data/  als Input?
* Game/OSM-Editor and simulation. Ab 14:30 - https://youtube.com/watch?v=LiIoE8ArACs&si=o4XecnOFtyhClmBp =>
    https://github.com/citybound/citybound

* Terrain onls?: https://www.reddit.com/r/openstreetmap/comments/1j1u2nd/still_working_on_osm_viewer_some_new_screenshots/?rdt=50604
* How to make building parts: https://github.com/louis-e/arnis
* What F4maps shows, and what not: https://demo.f4map.com/#lat=48.8043521&lon=9.5300202&zoom=21&camera.theta=50.493&camera.phi=-13.465
