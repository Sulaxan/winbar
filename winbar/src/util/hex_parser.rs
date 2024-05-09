use anyhow::{bail, Result};
use regex::Regex;

#[derive(Clone, Debug, PartialEq)]
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
    let re = Regex::new(
        "^#?(?<r>[0-9A-F]{2})(?<g>[0-9A-F]{2})(?<b>[0-9A-F]{2})(?<alpha>[0-9A-F]{2})?$",
    )?;
    match re.captures(hex) {
        Some(captures) => {
            let r = captures.name("r").unwrap().as_str();
            let g = captures.name("g").unwrap().as_str();
            let b = captures.name("b").unwrap().as_str();
            let alpha = match captures.name("alpha") {
                Some(mat) => Some(parse_hex(mat.as_str())?),
                _ => None,
            };

            Ok(HexColor {
                r: parse_hex(r)?,
                g: parse_hex(g)?,
                b: parse_hex(b)?,
                alpha,
            })
        }
        None => bail!("Invalid hex color: {}", hex),
    }
}

/// Parses a hex digit, returning it's decimal form. Valid hex digits are in the following form:
/// /([0-9A-F])+/.
///
/// Note that this function currently does not support negative digits.
pub fn parse_hex(hex: &str) -> Result<u32> {
    let mut current_exp = hex.len() as u32;
    let mut hex_val = 0u32;
    for c in hex.chars() {
        let val = match c.to_ascii_uppercase() {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'A' => 10,
            'B' => 11,
            'C' => 12,
            'D' => 13,
            'E' => 14,
            'F' => 15,
            _ => bail!("Invalid hex digit {} in {}", c, hex),
        };
        hex_val += val * 16u32.pow(current_exp - 1);
        current_exp -= 1;
    }

    Ok(hex_val)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_color_no_hash_symbol_works() {
        let hex = "000000";
        assert_eq!(
            parse_color(hex).unwrap(),
            HexColor {
                r: 0,
                g: 0,
                b: 0,
                alpha: None,
            }
        )
    }

    #[test]
    fn parse_color_works() {
        let hex = "#FFFFFF";
        assert_eq!(
            parse_color(hex).unwrap(),
            HexColor {
                r: 255,
                g: 255,
                b: 255,
                alpha: None,
            }
        )
    }

    #[test]
    fn parse_color_with_alpha_works() {
        let hex = "#112233AF";
        assert_eq!(
            parse_color(hex).unwrap(),
            HexColor {
                r: 17,
                g: 34,
                b: 51,
                alpha: Some(175),
            }
        )
    }

    #[test]
    fn parse_hex_works() {
        let hex = "FF";
        assert_eq!(parse_hex(hex).unwrap(), 255);
    }
}
