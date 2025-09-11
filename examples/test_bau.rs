// https://docs.rs/geo/latest/geo/
// https://github.com/georust/geo
// https://www.geeksforgeeks.org/dsa/how-to-check-if-a-given-point-lies-inside-a-polygon/

//use i_float::float::compatible::FloatPointCompatible;

// primitives
use geo::{TriangulateEarcut, polygon};

fn main() {
    // An L shape
    let polygon = polygon![
                            //                          5   .   .   4
                            //
        (x: 0.0, y: 0.0),   // 0                        .   .   .   3   2
        (x: 4.0, y: 0.0),   // 1
        (x: 4.0, y: 3.0),   // 2                        .   .   .   .   .
        (x: 3.0, y: 3.0),   // 3
        (x: 3.0, y: 4.0),   // 4                        .   .   .   .   .
        (x: 0.0, y: 4.0),   // 5
        (x: 0.0, y: 0.0),   // 6 = 0 is not needed      0   .   .   .   1
    ];

    // triangles: RawTriangulation { vertices: [0.0, 0.0, 4.0, 0.0, 4.0, 1.0, 1.0, 1.0, 1.0, 4.0, 0.0, 4.0, 0.0, 0.0],
    // triangle_indices: [0, 1, 2,  3, 4, 5,  0, 2, 3,  3, 5, 0] }
    //                   [5, 0, 1,  1, 2, 3,  3, 4, 5,  5, 1, 3]

    let polygon2 = polygon![
       /* 0 */ (x:-27.419066613139673  ,y:-31.33583999984012),
       /* 1 */ (x:-13.03044527993139   ,y:-28.8800880003555),
       /* 2 */ (x:-13.74374087546301   ,y:-24.735312000082104),
       /* 3 */ (x:-2.5830298123234026  ,y:-22.82404799985443),
       /* 4 */ (x:-1.9273667037013107  ,y:-26.657688000439634),
       /* 5 */ (x:11.819976488099567   ,y:-24.31305599988491),
       /* 6 */ (x:11.099457284650905   ,y:-20.112720000541344),
       /* 7 */ (x:22.411457042288635   ,y:-18.179232000053958),
       /* 8 */ (x:22.894210471500347   ,y:-20.990568000535745),
       /* 9 */ (x:36.266863811952206   ,y:-18.701495999840745),
       /*10 */ (x:27.706951004079286   ,y: 31.33583999984012),
       /*11 */ (x:-36.266602361617025  ,y: 20.390519999839967),
    ];

    // 11, 0, 1, 3, 4, 5, 7, 8, 9, 9, 10, 11, 11, 1, 2, 3, 5, 6, 7, 9, 11, 11, 2, 3, 6, 7, 11, 11, 3, 6

    let triangles = polygon2.earcut_triangles_raw();
    println!("triangles-2: {:?}", triangles);

    let triangles = polygon.earcut_triangles_raw();
    println!("triangles: {:?}", triangles);

    assert_eq!(
        triangles.triangle_indices,
        vec![5, 0, 1, 1, 2, 3, 3, 4, 5, 5, 1, 3]
    );
}
