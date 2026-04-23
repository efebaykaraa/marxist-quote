pub fn parse_color_to_rgba(hex: &str) -> (f64, f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 || hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f64 / 255.0
        } else {
            1.0
        };
        (r, g, b, a)
    } else {
        (1.0, 1.0, 1.0, 1.0)
    }
}

pub fn rgba_to_hex(r: f64, g: f64, b: f64, a: f64) -> String {
    format!("#{:02x}{:02x}{:02x}{:02x}", 
        (r * 255.0) as u8, 
        (g * 255.0) as u8, 
        (b * 255.0) as u8,
        (a * 255.0) as u8)
}
