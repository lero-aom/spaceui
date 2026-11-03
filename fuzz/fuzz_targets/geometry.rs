#![no_main]

use libfuzzer_sys::fuzz_target;
use spaceui::{
    egui::{Color32, Mesh, Rect, pos2},
    paint,
};

fn check_mesh(mesh: &Mesh) {
    assert!(mesh.vertices.iter().all(|vertex| vertex.pos.is_finite()));
    assert!(mesh.vertices.iter().all(|vertex| vertex.uv.is_finite()));
    assert!(mesh.indices.iter().all(|index| (*index as usize) < mesh.vertices.len()));
    assert_eq!(mesh.indices.len() % 3, 0);
}

fuzz_target!(|data: &[u8]| {
    // Raw bits deliberately include NaN, infinities, subnormals, signed zero,
    // enormous finite values and reversed rectangles. Cap the polygon size so
    // an input cannot exhaust the fuzz worker's allocation budget.
    let values: Vec<_> = data
        .chunks_exact(4)
        .take(71)
        .map(|bytes| f32::from_le_bytes(bytes.try_into().expect("exact four-byte chunk")))
        .collect();
    if values.len() < 7 {
        return;
    }
    let rect = Rect::from_min_max(pos2(values[0], values[1]), pos2(values[2], values[3]));
    let radius = values[4];
    let density = values[5];
    let feather = values[6];
    let outline = paint::rounded_rect_points(rect, radius, density);
    assert!(outline.iter().all(|point| point.is_finite()));
    assert!(outline.len() <= 196, "rounded-rectangle subdivision must be bounded");
    check_mesh(&paint::gradient_mesh(rect, Color32::WHITE, Color32::BLACK, radius, density));
    let points: Vec<_> = values[7..].chunks_exact(2).map(|xy| pos2(xy[0], xy[1])).collect();
    check_mesh(&paint::polygon_mesh(&points, feather, |_| Color32::WHITE));
});
