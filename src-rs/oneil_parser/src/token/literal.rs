//! Provides parsers for literal values in the Oneil language.
//!
//! This module contains parsers for numeric and string literals. The numeric parser
//! supports integers and floating point numbers with optional signs and exponents.
//! The string parser handles double-quoted string literals.

use nom::{
    AsChar, Parser as NomParser,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{digit1, one_of, satisfy},
    combinator::{eof, opt, peek},
};

use crate::token::{
    InputSpan, Result,
    error::{ErrorHandlingParser, TokenError},
    util::{Token, token},
};

/// Parses a number literal, supporting optional sign, decimal, and exponent.
///
/// All valid numbers should be parsed correctly because they conform to the
/// grammar noted here:
/// <https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr-for-f64>
///
/// Therefore, when this parser is used, we can use the following pattern to convert it to an f64:
/// ```ignore
/// let parse_result = n.lexeme().parse::<f64>();
/// let parse_result = parse_result.expect("all valid numbers should parse correctly");
/// ```
///
/// The parser handles the following number formats:
/// - Integers: `42`, `-17`, `+123`
/// - Decimals: `3.1415`, `-2.5`, `+0.1`, `.123`
/// - Exponents: `2.5e10`, `-1.2E-3`, `1e+5`
/// - Combined: `-1.23e-4`, `+5.67E+2`
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the parsed number literal.
pub fn number(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    // Optional sign (+ or -) at the beginning
    let opt_sign_1 = opt(one_of("+-"));
    let opt_sign_2 = opt(one_of("+-"));

    // Parse the number part: either digits followed by optional decimal, or just decimal
    let decimal_part = |input| {
        let (rest, decimal_point_span) = tag(".").parse(input)?;
        let (rest, _) = digit1
            .or_fail_with(TokenError::invalid_decimal_part(decimal_point_span))
            .parse(rest)?;
        Ok((rest, ()))
    };

    let number_part = |input| {
        // Try digits first (with optional decimal)
        let with_digits = |input| {
            let (rest, _) = digit1.parse(input)?;
            let (rest, _) = opt(decimal_part).parse(rest)?;
            Ok((rest, ()))
        };

        // Try decimal without digits
        let decimal_only = decimal_part;

        alt((with_digits, decimal_only)).parse(input)
    };

    // Optional exponent part (e.g., "e10", "E-3")
    let opt_exponent = opt(|input| {
        let (rest, e_span) = tag("e").or(tag("E")).parse(input)?;
        let (rest, _) = opt(one_of("+-")).parse(rest)?;
        let (rest, _) = digit1
            .or_fail_with(TokenError::invalid_exponent_part(e_span))
            .parse(rest)?;
        Ok((rest, ()))
    });

    // Parse `inf` literal (not keyword)
    let inf_literal = |input| {
        let (rest, _) = tag("inf").parse(input)?;
        let next_char_is_not_ident_char =
            peek(satisfy(|c: char| !c.is_alphanumeric() && c != '_')).map(|_| ());
        let reached_end_of_file = eof.map(|_| ());
        let (rest, ()) = next_char_is_not_ident_char
            .or(reached_end_of_file)
            .parse(rest)?;
        Ok((rest, ()))
    };

    // Parse a number literal
    let number_literal = alt((
        (opt_sign_1, number_part, opt_exponent).map(|_| ()),
        (opt_sign_2, inf_literal).map(|_| ()),
    ));

    token(number_literal, TokenError::expected_number).parse(input)
}

/// Parses a string literal delimited by single quotes.
///
/// String literals in Oneil are delimited by single quotes (`'`) and can contain
/// any characters except single quotes and newlines. The parser does not support
/// escape sequences, so backslashes are treated as literal characters.
///
/// Examples of valid strings:
/// - `'hello'`
/// - `'foo bar'`
/// - `'123'`
///
/// The parser will fail if:
/// - The string is not properly closed with a single quote
/// - The string contains a newline character
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the parsed string literal including the quotes.
pub fn string(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        |input| {
            let (rest, open_quote_span) = tag("\'").parse(input)?;
            let (rest, _) = take_while(|c: char| c != '\'' && c != '\n').parse(rest)?;
            let (rest, _) = tag("'")
                .or_fail_with(TokenError::unclosed_string(open_quote_span))
                .parse(rest)?;
            Ok((rest, ()))
        },
        TokenError::expected_string,
    )
    .parse(input)
}

/// Parses a unit one literal, which is simply the character "1".
///
/// A unit one represents a unitless 1, usually used for units like 1/s.
/// According to the grammar, this is just the literal "1" character.
///
/// Examples of valid unit ones:
/// - `1`
///
/// The parser will fail if:
/// - The input doesn't start with "1"
/// - The input is empty
/// - The "1" is followed by additional digits (e.g., "123" should fail)
///
/// # Arguments
///
/// * `input` - The input span to parse
///
/// # Returns
///
/// Returns a token containing the parsed unit one literal.
pub fn unit_one(input: InputSpan<'_>) -> Result<'_, Token<'_>, TokenError> {
    token(
        |input| {
            let next_char_is_not_digit = satisfy(|c: char| !c.is_dec_digit()).map(|_| ());
            let is_at_end_of_file = eof.map(|_| ());

            let (rest, _) = tag("1").parse(input)?;
            // Ensure the next character is not a digit
            let (rest, ()) = peek(next_char_is_not_digit.or(is_at_end_of_file)).parse(rest)?;
            Ok((rest, ()))
        },
        TokenError::expected_unit_one,
    )
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Config, InputSpan,
        token::error::{ExpectKind, IncompleteKind, TokenErrorKind},
    };

    mod number {

        use super::*;

        // Success cases
        #[test]
        fn integer() {
            let input = InputSpan::new_extra("42 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse integer");
            assert_eq!(matched.lexeme(), "42");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn negative_integer() {
            let input = InputSpan::new_extra("-17 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse negative integer");
            assert_eq!(matched.lexeme(), "-17");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn decimal() {
            let input = InputSpan::new_extra("3.1415 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse decimal");
            assert_eq!(matched.lexeme(), "3.1415");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn exponent() {
            let input = InputSpan::new_extra("2.5e10 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse exponent");
            assert_eq!(matched.lexeme(), "2.5e10");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn negative_exponent() {
            let input = InputSpan::new_extra("-1.2E-3 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse negative exponent");
            assert_eq!(matched.lexeme(), "-1.2E-3");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn multiple_decimal_points() {
            let input = InputSpan::new_extra("123.456.789", Config::default());
            let (rest, matched) = number(input).expect("should parse first decimal part");
            assert_eq!(matched.lexeme(), "123.456");
            assert_eq!(rest.fragment(), &".789");
        }

        #[test]
        fn multiple_exponents() {
            let input = InputSpan::new_extra("123e10e5", Config::default());
            let (rest, matched) = number(input).expect("should parse first exponent part");
            assert_eq!(matched.lexeme(), "123e10");
            assert_eq!(rest.fragment(), &"e5");
        }

        #[test]
        fn exponent_before_decimal() {
            let input = InputSpan::new_extra("123e5.456", Config::default());
            let (rest, matched) = number(input).expect("should parse exponent part");
            assert_eq!(matched.lexeme(), "123e5");
            assert_eq!(rest.fragment(), &".456");
        }

        #[test]
        fn invalid_exponent_letter() {
            let input = InputSpan::new_extra("123f5", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"f5");
        }

        #[test]
        fn invalid_exponent_letter_uppercase() {
            let input = InputSpan::new_extra("123F5", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"F5");
        }

        #[test]
        fn with_letters_mixed() {
            let input = InputSpan::new_extra("123abc", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"abc");
        }

        #[test]
        fn with_symbols_mixed() {
            let input = InputSpan::new_extra("123+456", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"+456");
        }

        #[test]
        fn leading_zeros() {
            let input = InputSpan::new_extra("00123 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse leading zeros");
            assert_eq!(matched.lexeme(), "00123");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn negative_zero() {
            let input = InputSpan::new_extra("-0 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse negative zero");
            assert_eq!(matched.lexeme(), "-0");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn positive_zero() {
            let input = InputSpan::new_extra("+0 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse positive zero");
            assert_eq!(matched.lexeme(), "+0");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn zero_decimal() {
            let input = InputSpan::new_extra("0.123 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse zero decimal");
            assert_eq!(matched.lexeme(), "0.123");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn zero_exponent() {
            let input = InputSpan::new_extra("123e0 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse zero exponent");
            assert_eq!(matched.lexeme(), "123e0");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn max_precision() {
            let input = InputSpan::new_extra("3.141592653589793 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse high precision");
            assert_eq!(matched.lexeme(), "3.141592653589793");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn large_exponent() {
            let input = InputSpan::new_extra("1e308 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse large exponent");
            assert_eq!(matched.lexeme(), "1e308");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn small_exponent() {
            let input = InputSpan::new_extra("1e-308 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse small exponent");
            assert_eq!(matched.lexeme(), "1e-308");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn decimal_without_digits() {
            let input = InputSpan::new_extra(".123 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse .123 successfully");
            assert_eq!(matched.lexeme(), ".123");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn decimal_without_digits_with_sign() {
            let input = InputSpan::new_extra("-.123 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse -.123 successfully");
            assert_eq!(matched.lexeme(), "-.123");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn decimal_without_digits_with_exponent() {
            let input = InputSpan::new_extra(".123e10 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse .123e10 successfully");
            assert_eq!(matched.lexeme(), ".123e10");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn decimal_without_digits_with_sign_and_exponent() {
            let input = InputSpan::new_extra("-.123e-10 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse -.123e-10 successfully");
            assert_eq!(matched.lexeme(), "-.123e-10");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn inf_literal() {
            let input = InputSpan::new_extra("inf rest", Config::default());
            let (rest, matched) = number(input).expect("should parse inf literal");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn positive_inf_literal() {
            let input = InputSpan::new_extra("+inf rest", Config::default());
            let (rest, matched) = number(input).expect("should parse +inf literal");
            assert_eq!(matched.lexeme(), "+inf");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn negative_inf_literal() {
            let input = InputSpan::new_extra("-inf rest", Config::default());
            let (rest, matched) = number(input).expect("should parse -inf literal");
            assert_eq!(matched.lexeme(), "-inf");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn inf_at_end_of_file() {
            let input = InputSpan::new_extra("inf", Config::default());
            let (rest, matched) = number(input).expect("should parse inf at end of file");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn positive_inf_at_end_of_file() {
            let input = InputSpan::new_extra("+inf", Config::default());
            let (rest, matched) = number(input).expect("should parse +inf at end of file");
            assert_eq!(matched.lexeme(), "+inf");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn negative_inf_at_end_of_file() {
            let input = InputSpan::new_extra("-inf", Config::default());
            let (rest, matched) = number(input).expect("should parse -inf at end of file");
            assert_eq!(matched.lexeme(), "-inf");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn inf_with_whitespace() {
            let input = InputSpan::new_extra("inf   rest", Config::default());
            let (rest, matched) = number(input).expect("should parse inf with trailing whitespace");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn inf_with_punctuation() {
            let input = InputSpan::new_extra("inf,", Config::default());
            let (rest, matched) = number(input).expect("should parse inf with comma");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &",");
        }

        #[test]
        fn inf_with_parentheses() {
            let input = InputSpan::new_extra("inf(", Config::default());
            let (rest, matched) = number(input).expect("should parse inf with opening parenthesis");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"(");
        }

        #[test]
        fn inf_with_symbols() {
            let input = InputSpan::new_extra("inf+", Config::default());
            let (rest, matched) = number(input).expect("should parse inf with plus symbol");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"+");
        }

        #[test]
        fn inf_with_newline() {
            let input = InputSpan::new_extra("inf\n", Config::default());
            let (rest, matched) = number(input).expect("should parse inf with newline");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"\n");
        }

        #[test]
        fn inf_with_tab() {
            let input = InputSpan::new_extra("inf\t", Config::default());
            let (rest, matched) = number(input).expect("should parse inf with tab");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn inf_with_carriage_return() {
            let input = InputSpan::new_extra("inf\r", Config::default());
            let (rest, matched) = number(input).expect("should parse inf with carriage return");
            assert_eq!(matched.lexeme(), "inf");
            assert_eq!(rest.fragment(), &"\r");
        }

        // Error cases
        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn letters_only() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn symbols_only() {
            let input = InputSpan::new_extra("+-", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn decimal_point_only() {
            let input = InputSpan::new_extra(".", Config::default());
            let res = number(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(InvalidDecimalPart), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::InvalidDecimalPart { .. })
            ));
        }

        #[test]
        fn exponent_only() {
            let input = InputSpan::new_extra("e", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn exponent_without_digits() {
            let input = InputSpan::new_extra("123e", Config::default());
            let res = number(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(InvalidExponentPart), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
            ));
        }

        #[test]
        fn exponent_with_sign_only() {
            let input = InputSpan::new_extra("123e+", Config::default());
            let res = number(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(InvalidExponentPart), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
            ));
        }

        #[test]
        fn exponent_with_sign_only_negative() {
            let input = InputSpan::new_extra("123e-", Config::default());
            let res = number(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(InvalidExponentPart), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
            ));
        }

        #[test]
        fn error_messages_are_specific() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = number(input);
            assert!(res.is_err(), "should fail with specific error");

            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError but got different error type: {res:?}");
            };

            assert!(
                matches!(token_error.kind, TokenErrorKind::Expect(ExpectKind::Number)),
                "error should be for Number"
            );
        }

        #[test]
        fn invalid_decimal_part_error() {
            let input = InputSpan::new_extra("123.", Config::default());
            let res = number(input);
            assert!(res.is_err(), "should fail on invalid decimal part");

            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError Failure but got different error type: {res:?}");
            };

            assert!(
                matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::InvalidDecimalPart { .. })
                ),
                "error should be for InvalidDecimalPart"
            );
        }

        #[test]
        fn invalid_exponent_part_error() {
            let input = InputSpan::new_extra("123e", Config::default());
            let res = number(input);
            assert!(res.is_err(), "should fail on invalid exponent part");

            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError Failure but got different error type: {res:?}");
            };

            assert!(
                matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
                ),
                "error should be for InvalidExponentPart"
            );
        }

        // Error cases for inf literals
        #[test]
        fn inf_with_letters_after() {
            let input = InputSpan::new_extra("infinity", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn inf_with_numbers_after() {
            let input = InputSpan::new_extra("inf123", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn inf_with_underscore_after() {
            let input = InputSpan::new_extra("inf_", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn partial_inf_match() {
            let input = InputSpan::new_extra("in", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn case_sensitive_inf() {
            let input = InputSpan::new_extra("INF", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn mixed_case_inf() {
            let input = InputSpan::new_extra("Inf", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }

        #[test]
        fn inf_not_at_start() {
            let input = InputSpan::new_extra("foo inf bar", Config::default());
            let res = number(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(Number), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::Number)
            ));
        }
    }

    mod string {
        use super::*;

        // Success cases
        #[test]
        fn simple() {
            let input = InputSpan::new_extra("'hello' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string");
            assert_eq!(matched.lexeme(), "'hello'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_spaces() {
            let input = InputSpan::new_extra("'foo bar' baz", Config::default());
            let (rest, matched) = string(input).expect("should parse string with spaces");
            assert_eq!(matched.lexeme(), "'foo bar'");
            assert_eq!(rest.fragment(), &"baz");
        }

        #[test]
        fn escape_sequences_not_supported() {
            let input = InputSpan::new_extra("'foo \\' bar", Config::default());
            let (rest, matched) =
                string(input).expect("should parse string (escape sequences not supported)");
            assert_eq!(matched.lexeme(), "'foo \\'");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn empty_string() {
            let input = InputSpan::new_extra("'' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse empty string");
            assert_eq!(matched.lexeme(), "''");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_carriage_return() {
            let input = InputSpan::new_extra("'hello\rworld' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with carriage return");
            assert_eq!(matched.lexeme(), "'hello\rworld'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_tab() {
            let input = InputSpan::new_extra("'hello\tworld' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with tab");
            assert_eq!(matched.lexeme(), "'hello\tworld'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_numbers() {
            let input = InputSpan::new_extra("'123 456' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with numbers");
            assert_eq!(matched.lexeme(), "'123 456'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_double_quotes() {
            let input = InputSpan::new_extra("'hello \"world\"' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with double quotes");
            assert_eq!(matched.lexeme(), "'hello \"world\"'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_backslashes() {
            let input = InputSpan::new_extra("'hello\\world' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with backslashes");
            assert_eq!(matched.lexeme(), "'hello\\world'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_whitespace_only() {
            let input = InputSpan::new_extra("'   ' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with whitespace only");
            assert_eq!(matched.lexeme(), "'   '");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn very_long() {
            let long_content = "a".repeat(1000);
            let input_str = format!("'{long_content}' rest");
            let input = InputSpan::new_extra(&input_str, Config::default());
            let (rest, matched) = string(input).expect("should parse very long string");
            let expected_lexeme = format!("'{long_content}'");
            assert_eq!(matched.lexeme(), expected_lexeme);
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn with_nested_single_quotes() {
            let input = InputSpan::new_extra("'hello'world' rest", Config::default());
            let (rest, matched) =
                string(input).expect("should parse string ending at first closing quote");
            assert_eq!(matched.lexeme(), "'hello'");
            assert_eq!(rest.fragment(), &"world' rest");
        }

        #[test]
        fn at_end_of_file() {
            let input = InputSpan::new_extra("'hello'", Config::default());
            let (rest, matched) = string(input).expect("should parse string at end of file");
            assert_eq!(matched.lexeme(), "'hello'");
            assert_eq!(rest.fragment(), &"");
        }

        // Error cases
        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = string(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(String), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::String)
            ));
        }

        #[test]
        fn no_opening_quote() {
            let input = InputSpan::new_extra("hello'", Config::default());
            let res = string(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(String), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::String)
            ));
        }

        #[test]
        fn no_closing_quote() {
            let input = InputSpan::new_extra("'hello", Config::default());
            let res = string(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(UnclosedString), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
            ));
        }

        #[test]
        fn with_newline() {
            let input = InputSpan::new_extra("'hello\nworld'", Config::default());
            let res = string(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(UnclosedString), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
            ));
        }

        #[test]
        fn with_mixed_whitespace() {
            let input = InputSpan::new_extra("' \t\n\r ' rest", Config::default());
            let res = string(input);
            assert!(
                res.is_err(),
                "should fail on string with mixed whitespace including newlines"
            );
        }

        #[test]
        fn unterminated_at_end_of_file() {
            let input = InputSpan::new_extra("'hello", Config::default());
            let res = string(input);
            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError::Incomplete(UnclosedString), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
            ));
        }

        #[test]
        fn error_messages_are_specific() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = string(input);
            assert!(res.is_err(), "should fail with specific error");

            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError but got different error type: {res:?}");
            };

            assert!(
                matches!(token_error.kind, TokenErrorKind::Expect(ExpectKind::String)),
                "error should be for String"
            );
        }

        #[test]
        fn unclosed_string_error() {
            let input = InputSpan::new_extra("'hello", Config::default());
            let res = string(input);
            assert!(res.is_err(), "should fail on unclosed string");

            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError Failure but got different error type: {res:?}");
            };

            assert!(
                matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
                ),
                "error should be for UnclosedString"
            );
        }

        #[test]
        fn unclosed_string_with_newline_error() {
            let input = InputSpan::new_extra("'hello\n", Config::default());
            let res = string(input);
            assert!(res.is_err(), "should fail on unclosed string with newline");

            let Err(nom::Err::Failure(token_error)) = res else {
                panic!("expected TokenError Failure but got different error type: {res:?}");
            };

            assert!(
                matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
                ),
                "error should be for UnclosedString"
            );
        }
    }

    mod unit_one {
        use super::*;

        // Success cases
        #[test]
        fn simple() {
            let input = InputSpan::new_extra("1 rest", Config::default());
            let (rest, matched) = unit_one(input).expect("should parse unit one");
            assert_eq!(matched.lexeme(), "1");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn at_end_of_file() {
            let input = InputSpan::new_extra("1", Config::default());
            let (rest, matched) = unit_one(input).expect("should parse unit one at end of file");
            assert_eq!(matched.lexeme(), "1");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn with_whitespace() {
            let input = InputSpan::new_extra("1  ", Config::default());
            let (rest, matched) = unit_one(input).expect("should parse unit one with whitespace");
            assert_eq!(matched.lexeme(), "1");
            assert_eq!(rest.fragment(), &"");
        }

        #[test]
        fn with_other_digits() {
            let input = InputSpan::new_extra("123", Config::default());
            let res = unit_one(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(UnitOne), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::UnitOne)
            ));
        }

        #[test]
        fn with_letters() {
            let input = InputSpan::new_extra("1abc", Config::default());
            let (rest, matched) = unit_one(input).expect("should parse unit one with letters");
            assert_eq!(matched.lexeme(), "1");
            assert_eq!(rest.fragment(), &"abc");
        }

        #[test]
        fn with_symbols() {
            let input = InputSpan::new_extra("1+2", Config::default());
            let (rest, matched) = unit_one(input).expect("should parse unit one with symbols");
            assert_eq!(matched.lexeme(), "1");
            assert_eq!(rest.fragment(), &"+2");
        }

        // Error cases
        #[test]
        fn empty_input() {
            let input = InputSpan::new_extra("", Config::default());
            let res = unit_one(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(UnitOne), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::UnitOne)
            ));
        }

        #[test]
        fn whitespace_only() {
            let input = InputSpan::new_extra("   ", Config::default());
            let res = unit_one(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(UnitOne), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::UnitOne)
            ));
        }

        #[test]
        fn other_digits() {
            let input = InputSpan::new_extra("2", Config::default());
            let res = unit_one(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(UnitOne), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::UnitOne)
            ));
        }

        #[test]
        fn letters_only() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = unit_one(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(UnitOne), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::UnitOne)
            ));
        }

        #[test]
        fn symbols_only() {
            let input = InputSpan::new_extra("+-", Config::default());
            let res = unit_one(input);
            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError::Expect(UnitOne), got {res:?}");
            };

            assert!(matches!(
                token_error.kind,
                TokenErrorKind::Expect(ExpectKind::UnitOne)
            ));
        }

        #[test]
        fn error_messages_are_specific() {
            let input = InputSpan::new_extra("abc", Config::default());
            let res = unit_one(input);
            assert!(res.is_err(), "should fail with specific error");

            let Err(nom::Err::Error(token_error)) = res else {
                panic!("expected TokenError but got different error type: {res:?}");
            };

            assert!(
                matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::UnitOne)
                ),
                "error should be for UnitOne"
            );
        }
    }
}
