//! Stable symbol identity shared by voxel entities, thumbnails and exported frames.
pub fn colour(symbol: char) -> ([u8; 3], &'static str) {
    match symbol {
        'a' => ([224, 163, 69], "amber"),
        'b' => ([66, 168, 232], "blue"),
        'c' => ([102, 204, 133], "green"),
        'A' => ([227, 102, 87], "red"),
        'B' => ([239, 221, 101], "yellow"),
        'C' => ([175, 136, 101], "copper"),
        'X' => ([186, 107, 222], "purple"),
        '(' => ([77, 209, 196], "teal"),
        ')' => ([237, 122, 176], "pink"),
        ' ' => ([61, 61, 71], "blank / charcoal"),
        _ => ([166, 179, 199], "other / silver"),
    }
}
