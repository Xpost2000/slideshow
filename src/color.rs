#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub fn hexadecimal_to_decimal(literal: &str) -> u8 {
    let mut result : u8 = 0;
    for (index, ch) in literal.chars().enumerate() {
        let index = (literal.len()-1) - index;
        result +=
            match ch {
                '0' => {0}, '1' => {1}, '2' => {2},
                '3' => {3}, '4' => {4}, '5' => {5},
                '6' => {6}, '7' => {7}, '8' => {8},
                '9' => {9}, 'A' => {10}, 'B' => {11},
                'C' => {12}, 'D' => {13}, 'E' => {14}, 'F' => {15},
                _ => { panic!("undefined character for hexadecimal literal!"); }
            } * 16_u8.pow(index as u32);
    }
    result
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color {r, g, b, a}
    }

    pub fn parse_hexadecimal_literal(hex: &str) -> Option<Color> {
        // TODO : This is not perfectly safe since it'll be fine up to
        // 255... This needs to be templated.

        if let Some('#') = hex.chars().nth(0) {
            let hex = &hex[1..];
            match hex.len() {
                // FFFFFF
                // FFFFFFFF
                6 | 8 => {
                    Some(Color::new(
                        hexadecimal_to_decimal(&hex[0..2]),
                        hexadecimal_to_decimal(&hex[2..4]),
                        hexadecimal_to_decimal(&hex[4..6]),
                        if hex.len() == 8 {
                            hexadecimal_to_decimal(&hex[6..8])
                        } else {
                            255
                        }
                    ))
                },
                _ => {
                    println!("Error parsing hexadecimal literal");
                    None
                }
            }
        } else {
            None
        }
    }
}
pub const COLOR_WHITE : Color = Color {r: 255, g: 255, b: 255, a: 255};
pub const COLOR_BLACK : Color = Color {r: 0, g: 0, b: 0, a: 0};
pub const COLOR_RIPE_LEMON : Color = Color {r: 247, g: 202, b: 24, a: 255};
