use anyhow::{anyhow, bail, Result};
use regex::Regex;

pub struct HexColor {
    /// The red component of the color.
    r: u32,
    /// The green component of the color.
    g: u32,
    /// The blue component of the color.
    b: u32,
    /// The alpha component of the color.
    alpha: Option<u32>,
}

/// Parses a hex string
pub fn parse_color(hex: &str) -> Result<HexColor> {
    let re =
        Regex::new("^#?<r>([0-9A-F]{2})<g>([0-9A-F]{2})<b>([0-9A-F]{2})<alpha>([0-9A-F]{2})?$")?;
    match re.captures(hex) {
        Some(captures) => {
            let r = captures.name("r").unwrap().as_str();
            let r = captures.name("g").unwrap().as_str();
            let b = captures.name("b").unwrap().as_str();
            let alpha = captures.name("alpha");
        }
        None => bail!("Invalid hex color: {}", hex),
    }
    todo!()
}

/// Parses a hex digit, returning it's decimal form.
///
/// Note that this function currently does not support negative digits.
pub fn parse_hex(hex: &str) -> Result<i32> {
    let re = Regex::new("^([0-9A-F])+$")?;
    match re.captures(hex) {
        Some(captures) => {
            // we subtract 2 since the first capture is the whole string (want to ignore this), and
            // the largest exponent we multiply by is 1 less than the length
            let mut current_exp = captures.len() as u32;
            let mut hex_val = 0;
            for mat in captures.iter().skip(1) {
                current_exp -= 1;

                match mat {
                    Some(mat) => {
                        let content = mat.as_str();
                        let val = match content.to_uppercase().as_str() {
                            "0" => 0,
                            "1" => 1,
                            "2" => 2,
                            "3" => 3,
                            "4" => 4,
                            "5" => 5,
                            "6" => 6,
                            "7" => 7,
                            "8" => 8,
                            "9" => 9,
                            "A" => 10,
                            "B" => 11,
                            "C" => 12,
                            "D" => 13,
                            "E" => 14,
                            "F" => 15,
                            other => bail!("Invalid hex digit {} in {}", other, hex),
                        };

                        println!("{}", val);

                        hex_val += val * 16i32.pow(current_exp);
                    }
                    None => bail!("Invalid hex capture: {}", hex),
                }
            }

            Ok(hex_val)
        }
        None => bail!("Invalid hex: {}", hex),
    }
}

mod test {
    use super::*;

    #[test]
    fn parse_hex_works() {
        let hex = "FF";
        assert_eq!(parse_hex(hex).unwrap(), 255);
    }
}
