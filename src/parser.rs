use std::str::FromStr;

use bigdecimal::BigDecimal;

pub enum EntryType {
    /// Entry is an expenditure
    Debit,
    /// Entry
    Credit,
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParseError {
    #[error("{0} is not a valid transaction type descriptor")]
    InvalidEntryType(String),
    #[error("{0} could not be parsed as a decimal")]
    InvalidDecimal(String),
    #[error("Expected description after '{0}'")]
    NoDescription(String),
}

pub struct Entry<'a> {
    pub typ: EntryType,
    pub decimal: BigDecimal,
    // TODO: rename to account?
    // TODO: make it optional?
    pub description: &'a str,
}

impl<'a> Entry<'a> {
    pub fn from_str(input: &'a str) -> ParseResult<Self> {
        let (typ, rest) = parse_entry_type(input)?;

        let (decimal, rest) = parse_decimal(rest)?;

        let description = parse_description(rest);

        Ok(Self {
            typ,
            decimal,
            description,
        })
    }
}

fn parse_entry_type(input: &str) -> ParseResult<(EntryType, &str)> {
    // Assumes input is trimmed
    debug_assert!(input == input.trim_start());
    // Assumes input is non-empty
    debug_assert!(input.len() > 0);

    let (first, rest) = input.split_at(1);

    match first {
        "+" => Ok((EntryType::Credit, rest)),
        "-" => Ok((EntryType::Debit, rest)),
        _ => Err(ParseError::InvalidEntryType(first.to_owned())),
    }
}

fn parse_decimal(input: &str) -> ParseResult<(BigDecimal, &str)> {
    let input = input.trim_start();

    fn parse_decimal<'a>(input: &'a str) -> Option<(&'a str, &'a str)> {
        let (decimal, rest) = input.split_once(' ')?;
        if rest.trim().is_empty() {
            None?
        }

        Some((decimal, rest))
    }

    let (decimal, rest) =
        parse_decimal(input).ok_or_else(|| ParseError::NoDescription(input.to_owned()))?;

    match BigDecimal::from_str(decimal).ok() {
        Some(decimal) => Ok((decimal, rest)),
        None => Err(ParseError::InvalidDecimal(decimal.to_owned())),
    }
}

#[inline]
fn parse_description(input: &str) -> &str {
    input.trim()
}

#[cfg(test)]
mod entry_parsing {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;

    use crate::parser::{parse_decimal, parse_description, ParseError};

    #[test]
    fn parses_valid_decimals_correctly() {
        let five = BigDecimal::from_str("5.00").unwrap();
        let approx_pi = BigDecimal::from_str("3.1415926535").unwrap();

        assert_eq!(parse_decimal(" 5.00 Test").unwrap(), (five.clone(), "Test"));

        assert_eq!(parse_decimal(" 5.00  Test").unwrap(), (five, " Test"));

        assert_eq!(
            parse_decimal("   3.1415926535 Pi").unwrap(),
            (approx_pi, "Pi")
        );
    }

    #[test]
    fn errs_on_invalid_decimals() {
        assert_eq!(
            parse_decimal("   NaN Pi").unwrap_err(),
            ParseError::InvalidDecimal("NaN".to_owned())
        );

        assert_eq!(
            parse_decimal("Hey 3.5").unwrap_err(),
            ParseError::InvalidDecimal("Hey".to_owned())
        );
    }

    #[test]
    fn errs_on_missing_description() {
        let approx_pi = "3.1415926535".to_string();
        let approx_pi_ws = "3.1415926535  ".to_string();

        assert_eq!(
            parse_decimal(&approx_pi).unwrap_err(),
            ParseError::NoDescription(approx_pi)
        );

        assert_eq!(
            parse_decimal("   3.1415926535  ").unwrap_err(),
            ParseError::NoDescription(approx_pi_ws)
        );
    }

    #[test]
    fn parses_descriptions_correctly() {
        assert_eq!("Petrobrás", parse_description("  Petrobrás"));
        assert_eq!("Petrobrás", parse_description("Petrobrás"));
        assert_eq!("Petrobrás", parse_description("Petrobrás   "));
        assert_eq!("Petrobrás", parse_description(" Petrobrás "));
    }
}
