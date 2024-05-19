use std::{marker::PhantomData, str::FromStr};

use anyhow::bail;
use regex::Regex;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use thiserror::Error;
use winbar::{color::Color, util::hex_parser};

#[derive(Clone, Default, Serialize, Deserialize)]
pub enum ColorConfig {
    Rgb {
        r: u32,
        g: u32,
        b: u32,
    },
    Rgba {
        r: u32,
        g: u32,
        b: u32,
        alpha: u32,
    },
    Hex(String),
    Transparent,
    /// Represents that the color should propogate to the next higher scopes's default color. This
    /// type does not exist past the config stage, and is thus invalid to convert into a `Color`.
    ///
    /// Note that this variant exists as a replacement for the None variant of Option<ColorConfig>
    /// to make it easier to parse `ColorConfig` from a string or map type using serde.
    #[default]
    Default,
}

impl ColorConfig {
    /// Transforms this `ColorConfig` into an `Option` of `Color`. Note that this method exists as
    /// not all `ColorConfig` variants exist within `Color`.
    pub fn into_color_option(self) -> Option<Color> {
        match self {
            ColorConfig::Default => None,
            _ => Some(self.into()),
        }
    }
}

impl From<ColorConfig> for Color {
    fn from(value: ColorConfig) -> Self {
        match value {
            ColorConfig::Rgb { r, g, b } => Color::Rgb { r, g, b },
            ColorConfig::Rgba { r, g, b, alpha } => Color::Rgba { r, g, b, alpha },
            ColorConfig::Hex(hex) => {
                let color = hex_parser::parse_color(&hex).unwrap();
                if let Some(alpha) = color.alpha() {
                    Color::Rgba {
                        r: *color.r(),
                        g: *color.g(),
                        b: *color.b(),
                        alpha: *alpha,
                    }
                } else {
                    Color::Rgb {
                        r: *color.r(),
                        g: *color.g(),
                        b: *color.b(),
                    }
                }
            }
            ColorConfig::Transparent => Color::Transparent,
            ColorConfig::Default => panic!("Cannot convert Default into valid Color"),
        }
    }
}

#[derive(Error, Debug)]
pub enum ColorParseError {
    #[error("could not compile regex")]
    CouldNotCompileRegex(#[from] regex::Error),
    #[error("specified color in invalid format: {0}")]
    InvalidColorFunctionSyntax(String),
    #[error("invalid color function: {0}")]
    InvalidFunction(String),
    #[error("error parsing color: {0}")]
    ParseError(#[from] anyhow::Error),
}

impl FromStr for ColorConfig {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new("(?<function>.+)\\((?<color>.*)\\)")
            .map_err(ColorParseError::CouldNotCompileRegex)?;
        match re.captures(s) {
            Some(captures) => {
                // capture groups should exist
                let function = captures.name("function").unwrap().as_str();
                let color = captures.name("color").unwrap().as_str();

                match function.to_lowercase().as_str() {
                    "rgb" | "rgba" => parse_inline_rgba(color).map_err(ColorParseError::ParseError),
                    "hex" => {
                        let hex =
                            hex_parser::parse_color(color).map_err(ColorParseError::ParseError)?;

                        Ok(match hex.alpha() {
                            Some(alpha) => ColorConfig::Rgba {
                                r: *hex.r(),
                                g: *hex.g(),
                                b: *hex.b(),
                                alpha: *alpha,
                            },
                            _ => ColorConfig::Rgb {
                                r: *hex.r(),
                                g: *hex.g(),
                                b: *hex.b(),
                            },
                        })
                    }
                    "transparent" => Ok(ColorConfig::Transparent),
                    _ => Err(ColorParseError::InvalidFunction(function.to_string())),
                }
            }
            _ => Err(ColorParseError::InvalidColorFunctionSyntax(s.to_string())),
        }
    }
}

/// Parses an inline rgb color.
///
/// Valid colors:
/// - 1,1,1
/// - 11, 22, 33
/// - 12 545 389
fn parse_inline_rgba(color: &str) -> anyhow::Result<ColorConfig> {
    let re = Regex::new("^(?<r>[0-9]{1,3}),?\\s?(?<g>[0-9]{1,3}),?\\s?(?<b>[0-9]{1,3})(,?\\s?(?<alpha>[0-9]{1,3}))?$")?;
    match re.captures(color) {
        Some(captures) => {
            // should be guaranteed r, g, b capture groups exist
            let r = captures.name("r").unwrap().as_str().parse()?;
            let g = captures.name("g").unwrap().as_str().parse()?;
            let b = captures.name("b").unwrap().as_str().parse()?;
            Ok(match captures.name("alpha") {
                Some(mat) => ColorConfig::Rgba {
                    r,
                    g,
                    b,
                    alpha: mat.as_str().parse()?,
                },
                _ => ColorConfig::Rgb { r, g, b },
            })
        }
        None => bail!("Invalid inline RGB sequence: {}", color),
    }
}

/// Parses a color as either a string or `ColorConfig`. Primarily used to deserialize a color in a
/// struct with serde.
///
/// Based on: https://serde.rs/string-or-struct.html
pub fn parse_string_or_color_config<'de, T, D>(deserialier: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = ColorParseError>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and forwards map types to
    // T's `Deserialize` impl. The `PhantomData` is to keep the compiler from complaining about T
    // being an unused generic type parameter. We need T in order to know the Value type for the
    // Visitor impl.
    struct StringOrColorConfig<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrColorConfig<T>
    where
        T: Deserialize<'de> + FromStr<Err = ColorParseError>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(FromStr::from_str(v).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }
    deserialier.deserialize_any(StringOrColorConfig(PhantomData))
}
