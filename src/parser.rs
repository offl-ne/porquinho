use std::cmp::Ordering;
use std::{ops::Not, str::FromStr};

use bigdecimal::BigDecimal;

#[derive(Debug, Clone, PartialOrd, PartialEq, Eq)]
pub enum OperationType {
    Withdraw,
    Deposit,
}

impl OperationType {
    pub fn name_and_symbol(&self) -> (&'static str, char) {
        match self {
            Self::Withdraw => ("take", '-'),
            Self::Deposit => ("put", '+'),
        }
    }
}

/// Put operations come before take operations
impl Ord for OperationType {
    fn cmp(&self, other: &Self) -> Ordering {
        use OperationType::*;

        match (self, other) {
            (Deposit, Withdraw) => Ordering::Less,
            (Deposit, Deposit) | (Withdraw, Withdraw) => Ordering::Equal,
            (Withdraw, Deposit) => Ordering::Greater,
        }
    }
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum ParseError {
    #[error("'{0}' is not a valid transaction type descriptor")]
    InvalidOperationType(String),
    #[error("'{0}' is not a valid month day")]
    InvalidDay(String),
    #[error("'{0}' could not be parsed as a decimal")]
    InvalidDecimal(String),
    #[error("Expected description after '{0}'")]
    NoDescription(String),
    #[error("Malformed operation: '{0}'")]
    Malformed(String),
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Operation {
    pub day: u8,
    pub kind: OperationType,
    pub amount: BigDecimal,
    // TODO: rename to account?
    // TODO: make it optional?
    pub description: String,
}

impl Operation {
    pub fn new(
        day: u8,
        kind: OperationType,
        amount: BigDecimal,
        description: impl ToString,
    ) -> Self {
        Self {
            day,
            kind,
            amount,
            description: description.to_string(),
        }
    }

    pub fn from_str(input: &str) -> ParseResult<Self> {
        let (day, rest) = parse_day(input)?;

        let (kind, rest) = parse_operation_type(rest)?;

        let (amount, rest) = parse_decimal(rest)?;

        let description = parse_description(rest).into();

        Ok(Self { day, kind, amount, description })
    }
}

fn parse_day(input: &str) -> ParseResult<(u8, &str)> {
    let (first, rest) = input
        .trim()
        .split_once(' ')
        .ok_or_else(|| ParseError::Malformed(input.to_owned()))?;

    // TODO: validate if this is a valid day?
    let day = first
        .parse()
        .map_err(|_| ParseError::InvalidDay(first.to_owned()))?;

    Ok((day, rest))
}

fn parse_operation_type(input: &str) -> ParseResult<(OperationType, &str)> {
    // Assumes input is trimmed
    debug_assert!(input == input.trim_start());
    // Assumes input is non-empty
    debug_assert!(input.is_empty().not());

    let (first, rest) = input.split_at(1);

    match first {
        "+" => Ok((OperationType::Deposit, rest)),
        "-" => Ok((OperationType::Withdraw, rest)),
        _ => Err(ParseError::InvalidOperationType(first.to_owned())),
    }
}

fn parse_decimal(input: &str) -> ParseResult<(BigDecimal, &str)> {
    let input = input.trim_start();

    fn parse_decimal(input: &str) -> Option<(&str, &str)> {
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
mod operation_parsing {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;

    use super::Operation;
    use crate::parser::{parse_decimal, parse_description, OperationType, ParseError};

    #[test]
    fn parses_operations_correctly() {
        let five = BigDecimal::from_str("5.00").unwrap();
        let six = BigDecimal::from_str("6.00").unwrap();

        assert_eq!(
            Operation::from_str("22 + 5.00 Salary").unwrap(),
            Operation {
                day: 22,
                kind: OperationType::Deposit,
                amount: five,
                description: "Salary".into()
            }
        );

        assert_eq!(
            Operation::from_str("12 - 6.000 Rent\n").unwrap(),
            Operation {
                day: 12,
                kind: OperationType::Withdraw,
                amount: six,
                description: "Rent".into()
            }
        );
    }

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
