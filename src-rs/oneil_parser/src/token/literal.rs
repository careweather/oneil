//! Provides parsers for literal values in the Oneil language.
//!
//! This module contains parsers for numeric and string literals. The numeric parser
//! supports integers and floating point numbers with optional signs and exponents.
//! The string parser handles double-quoted string literals.

use nom::{
    Parser as _,
    bytes::complete::{tag, take_while},
    character::complete::{digit1, one_of},
    combinator::opt,
};

use crate::token::{
    Result, Span,
    error::{ErrorHandlingParser, TokenError},
    util::{Token, token},
};

/// Parses a number literal, supporting optional sign, decimal, and exponent.
///
/// All valid numbers should be parsed correctly because they conform to the
/// grammar noted here:
/// https://doc.rust-lang.org/std/primitive.f64.html#impl-FromStr-for-f64
///
/// Therefore, when this parser is used, we can use the following pattern to convert it to an f64:
/// ```ignore
/// let parse_result = n.lexeme().parse::<f64>();
/// let parse_result = parse_result.expect("all valid numbers should parse correctly");
/// ```
///
/// The parser handles the following number formats:
/// - Integers: `42`, `-17`, `+123`
/// - Decimals: `3.1415`, `-2.5`, `+0.1`
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
pub fn number(input: Span) -> Result<Token, TokenError> {
    // Optional sign (+ or -) at the beginning
    let opt_sign = opt(one_of("+-"));

    // Required sequence of digits
    let digit = digit1;

    // Optional decimal part (e.g., ".1415")
    let opt_decimal = opt(|input| -> Result<_, TokenError> {
        let (rest, decimal_point_span) = tag(".").parse(input)?;
        let (rest, _) = digit1
            .or_fail_with(TokenError::invalid_decimal_part(decimal_point_span))
            .parse(rest)?;
        Ok((rest, ()))
    });

    // Optional exponent part (e.g., "e10", "E-3")
    let opt_exponent = opt(|input| {
        let (rest, e_span) = tag("e").or(tag("E")).parse(input)?;
        let (rest, _) = opt(one_of("+-")).parse(rest)?;
        let (rest, _) = digit1
            .or_fail_with(TokenError::invalid_exponent_part(e_span))
            .parse(rest)?;
        Ok((rest, ()))
    });

    token(
        (opt_sign, digit, opt_decimal, opt_exponent),
        TokenError::expected_number,
    )
    .parse(input)
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
pub fn string(input: Span) -> Result<Token, TokenError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Config, Span,
        token::error::{ExpectKind, IncompleteKind, TokenErrorKind},
    };

    mod number_tests {

        use super::*;

        // Success cases
        #[test]
        fn test_integer() {
            let input = Span::new_extra("42 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse integer");
            assert_eq!(matched.lexeme(), "42");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_negative_integer() {
            let input = Span::new_extra("-17 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse negative integer");
            assert_eq!(matched.lexeme(), "-17");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_decimal() {
            let input = Span::new_extra("3.1415 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse decimal");
            assert_eq!(matched.lexeme(), "3.1415");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_exponent() {
            let input = Span::new_extra("2.5e10 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse exponent");
            assert_eq!(matched.lexeme(), "2.5e10");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_negative_exponent() {
            let input = Span::new_extra("-1.2E-3 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse negative exponent");
            assert_eq!(matched.lexeme(), "-1.2E-3");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_multiple_decimal_points() {
            let input = Span::new_extra("123.456.789", Config::default());
            let (rest, matched) = number(input).expect("should parse first decimal part");
            assert_eq!(matched.lexeme(), "123.456");
            assert_eq!(rest.fragment(), &".789");
        }

        #[test]
        fn test_multiple_exponents() {
            let input = Span::new_extra("123e10e5", Config::default());
            let (rest, matched) = number(input).expect("should parse first exponent part");
            assert_eq!(matched.lexeme(), "123e10");
            assert_eq!(rest.fragment(), &"e5");
        }

        #[test]
        fn test_exponent_before_decimal() {
            let input = Span::new_extra("123e5.456", Config::default());
            let (rest, matched) = number(input).expect("should parse exponent part");
            assert_eq!(matched.lexeme(), "123e5");
            assert_eq!(rest.fragment(), &".456");
        }

        #[test]
        fn test_invalid_exponent_letter() {
            let input = Span::new_extra("123f5", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"f5");
        }

        #[test]
        fn test_invalid_exponent_letter_uppercase() {
            let input = Span::new_extra("123F5", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"F5");
        }

        #[test]
        fn test_with_letters_mixed() {
            let input = Span::new_extra("123abc", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"abc");
        }

        #[test]
        fn test_with_symbols_mixed() {
            let input = Span::new_extra("123+456", Config::default());
            let (rest, matched) = number(input).expect("should parse digits only");
            assert_eq!(matched.lexeme(), "123");
            assert_eq!(rest.fragment(), &"+456");
        }

        #[test]
        fn test_leading_zeros() {
            let input = Span::new_extra("00123 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse leading zeros");
            assert_eq!(matched.lexeme(), "00123");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_negative_zero() {
            let input = Span::new_extra("-0 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse negative zero");
            assert_eq!(matched.lexeme(), "-0");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_positive_zero() {
            let input = Span::new_extra("+0 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse positive zero");
            assert_eq!(matched.lexeme(), "+0");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_zero_decimal() {
            let input = Span::new_extra("0.123 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse zero decimal");
            assert_eq!(matched.lexeme(), "0.123");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_zero_exponent() {
            let input = Span::new_extra("123e0 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse zero exponent");
            assert_eq!(matched.lexeme(), "123e0");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_max_precision() {
            let input = Span::new_extra("3.141592653589793 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse high precision");
            assert_eq!(matched.lexeme(), "3.141592653589793");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_large_exponent() {
            let input = Span::new_extra("1e308 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse large exponent");
            assert_eq!(matched.lexeme(), "1e308");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_small_exponent() {
            let input = Span::new_extra("1e-308 rest", Config::default());
            let (rest, matched) = number(input).expect("should parse small exponent");
            assert_eq!(matched.lexeme(), "1e-308");
            assert_eq!(rest.fragment(), &"rest");
        }

        // Error cases
        #[test]
        fn test_empty_input() {
            let input = Span::new_extra("", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Number)
                )),
                _ => panic!("expected TokenError::Expect(Number), got {:?}", res),
            }
        }

        #[test]
        fn test_whitespace_only() {
            let input = Span::new_extra("   ", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Number)
                )),
                _ => panic!("expected TokenError::Expect(Number), got {:?}", res),
            }
        }

        #[test]
        fn test_letters_only() {
            let input = Span::new_extra("abc", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Number)
                )),
                _ => panic!("expected TokenError::Expect(Number), got {:?}", res),
            }
        }

        #[test]
        fn test_symbols_only() {
            let input = Span::new_extra("+-", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Number)
                )),
                _ => panic!("expected TokenError::Expect(Number), got {:?}", res),
            }
        }

        #[test]
        fn test_decimal_point_only() {
            let input = Span::new_extra(".", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Number)
                )),
                _ => panic!("expected TokenError::Expect(Number), got {:?}", res),
            }
        }

        #[test]
        fn test_decimal_without_digits() {
            let input = Span::new_extra(".123", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Number)
                )),
                _ => panic!("expected TokenError::Expect(Number), got {:?}", res),
            }
        }

        #[test]
        fn test_exponent_only() {
            let input = Span::new_extra("e", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::Number)
                )),
                _ => panic!("expected TokenError::Expect(Number), got {:?}", res),
            }
        }

        #[test]
        fn test_exponent_without_digits() {
            let input = Span::new_extra("123e", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Failure(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
                )),
                _ => panic!(
                    "expected TokenError::Incomplete(InvalidExponentPart), got {:?}",
                    res
                ),
            }
        }

        #[test]
        fn test_exponent_with_sign_only() {
            let input = Span::new_extra("123e+", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Failure(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
                )),
                _ => panic!(
                    "expected TokenError::Incomplete(InvalidExponentPart), got {:?}",
                    res
                ),
            }
        }

        #[test]
        fn test_exponent_with_sign_only_negative() {
            let input = Span::new_extra("123e-", Config::default());
            let res = number(input);
            match res {
                Err(nom::Err::Failure(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
                )),
                _ => panic!(
                    "expected TokenError::Incomplete(InvalidExponentPart), got {:?}",
                    res
                ),
            }
        }

        #[test]
        fn test_error_messages_are_specific() {
            let input = Span::new_extra("abc", Config::default());
            let res = number(input);
            assert!(res.is_err(), "should fail with specific error");

            if let Err(nom::Err::Error(token_error)) = res {
                assert!(
                    matches!(token_error.kind, TokenErrorKind::Expect(ExpectKind::Number)),
                    "error should be for Number"
                );
            } else {
                panic!(
                    "expected TokenError but got different error type: {:?}",
                    res
                );
            }
        }

        #[test]
        fn test_invalid_decimal_part_error() {
            let input = Span::new_extra("123.", Config::default());
            let res = number(input);
            assert!(res.is_err(), "should fail on invalid decimal part");

            if let Err(nom::Err::Failure(token_error)) = res {
                assert!(
                    matches!(
                        token_error.kind,
                        TokenErrorKind::Incomplete(IncompleteKind::InvalidDecimalPart { .. })
                    ),
                    "error should be for InvalidDecimalPart"
                );
            } else {
                panic!(
                    "expected TokenError Failure but got different error type: {:?}",
                    res
                );
            }
        }

        #[test]
        fn test_invalid_exponent_part_error() {
            let input = Span::new_extra("123e", Config::default());
            let res = number(input);
            assert!(res.is_err(), "should fail on invalid exponent part");

            if let Err(nom::Err::Failure(token_error)) = res {
                assert!(
                    matches!(
                        token_error.kind,
                        TokenErrorKind::Incomplete(IncompleteKind::InvalidExponentPart { .. })
                    ),
                    "error should be for InvalidExponentPart"
                );
            } else {
                panic!(
                    "expected TokenError Failure but got different error type: {:?}",
                    res
                );
            }
        }
    }

    mod string_tests {
        use super::*;

        // Success cases
        #[test]
        fn test_simple() {
            let input = Span::new_extra("'hello' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string");
            assert_eq!(matched.lexeme(), "'hello'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_spaces() {
            let input = Span::new_extra("'foo bar' baz", Config::default());
            let (rest, matched) = string(input).expect("should parse string with spaces");
            assert_eq!(matched.lexeme(), "'foo bar'");
            assert_eq!(rest.fragment(), &"baz");
        }

        #[test]
        fn test_escape_sequences_not_supported() {
            let input = Span::new_extra("'foo \\' bar", Config::default());
            let (rest, matched) =
                string(input).expect("should parse string (escape sequences not supported)");
            assert_eq!(matched.lexeme(), "'foo \\'");
            assert_eq!(rest.fragment(), &"bar");
        }

        #[test]
        fn test_empty_string() {
            let input = Span::new_extra("'' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse empty string");
            assert_eq!(matched.lexeme(), "''");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_carriage_return() {
            let input = Span::new_extra("'hello\rworld' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with carriage return");
            assert_eq!(matched.lexeme(), "'hello\rworld'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_tab() {
            let input = Span::new_extra("'hello\tworld' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with tab");
            assert_eq!(matched.lexeme(), "'hello\tworld'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_numbers() {
            let input = Span::new_extra("'123 456' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with numbers");
            assert_eq!(matched.lexeme(), "'123 456'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_double_quotes() {
            let input = Span::new_extra("'hello \"world\"' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with double quotes");
            assert_eq!(matched.lexeme(), "'hello \"world\"'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_backslashes() {
            let input = Span::new_extra("'hello\\world' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with backslashes");
            assert_eq!(matched.lexeme(), "'hello\\world'");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_whitespace_only() {
            let input = Span::new_extra("'   ' rest", Config::default());
            let (rest, matched) = string(input).expect("should parse string with whitespace only");
            assert_eq!(matched.lexeme(), "'   '");
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_very_long() {
            let long_content = "a".repeat(1000);
            let input_str = format!("'{}' rest", long_content);
            let input = Span::new_extra(&input_str, Config::default());
            let (rest, matched) = string(input).expect("should parse very long string");
            let expected_lexeme = format!("'{}'", long_content);
            assert_eq!(matched.lexeme(), expected_lexeme);
            assert_eq!(rest.fragment(), &"rest");
        }

        #[test]
        fn test_with_nested_single_quotes() {
            let input = Span::new_extra("'hello'world' rest", Config::default());
            let (rest, matched) =
                string(input).expect("should parse string ending at first closing quote");
            assert_eq!(matched.lexeme(), "'hello'");
            assert_eq!(rest.fragment(), &"world' rest");
        }

        #[test]
        fn test_at_end_of_file() {
            let input = Span::new_extra("'hello'", Config::default());
            let (rest, matched) = string(input).expect("should parse string at end of file");
            assert_eq!(matched.lexeme(), "'hello'");
            assert_eq!(rest.fragment(), &"");
        }

        // Error cases
        #[test]
        fn test_empty_input() {
            let input = Span::new_extra("", Config::default());
            let res = string(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::String)
                )),
                _ => panic!("expected TokenError::Expect(String), got {:?}", res),
            }
        }

        #[test]
        fn test_no_opening_quote() {
            let input = Span::new_extra("hello'", Config::default());
            let res = string(input);
            match res {
                Err(nom::Err::Error(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Expect(ExpectKind::String)
                )),
                _ => panic!("expected TokenError::Expect(String), got {:?}", res),
            }
        }

        #[test]
        fn test_no_closing_quote() {
            let input = Span::new_extra("'hello", Config::default());
            let res = string(input);
            match res {
                Err(nom::Err::Failure(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
                )),
                _ => panic!(
                    "expected TokenError::Incomplete(UnclosedString), got {:?}",
                    res
                ),
            }
        }

        #[test]
        fn test_with_newline() {
            let input = Span::new_extra("'hello\nworld'", Config::default());
            let res = string(input);
            match res {
                Err(nom::Err::Failure(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
                )),
                _ => panic!(
                    "expected TokenError::Incomplete(UnclosedString), got {:?}",
                    res
                ),
            }
        }

        #[test]
        fn test_with_mixed_whitespace() {
            let input = Span::new_extra("' \t\n\r ' rest", Config::default());
            let res = string(input);
            assert!(
                res.is_err(),
                "should fail on string with mixed whitespace including newlines"
            );
        }

        #[test]
        fn test_unterminated_at_end_of_file() {
            let input = Span::new_extra("'hello", Config::default());
            let res = string(input);
            match res {
                Err(nom::Err::Failure(token_error)) => assert!(matches!(
                    token_error.kind,
                    TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
                )),
                _ => panic!(
                    "expected TokenError::Incomplete(UnclosedString), got {:?}",
                    res
                ),
            }
        }

        #[test]
        fn test_error_messages_are_specific() {
            let input = Span::new_extra("abc", Config::default());
            let res = string(input);
            assert!(res.is_err(), "should fail with specific error");

            if let Err(nom::Err::Error(token_error)) = res {
                assert!(
                    matches!(token_error.kind, TokenErrorKind::Expect(ExpectKind::String)),
                    "error should be for String"
                );
            } else {
                panic!(
                    "expected TokenError but got different error type: {:?}",
                    res
                );
            }
        }

        #[test]
        fn test_unclosed_string_error() {
            let input = Span::new_extra("'hello", Config::default());
            let res = string(input);
            assert!(res.is_err(), "should fail on unclosed string");

            if let Err(nom::Err::Failure(token_error)) = res {
                assert!(
                    matches!(
                        token_error.kind,
                        TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
                    ),
                    "error should be for UnclosedString"
                );
            } else {
                panic!(
                    "expected TokenError Failure but got different error type: {:?}",
                    res
                );
            }
        }

        #[test]
        fn test_unclosed_string_with_newline_error() {
            let input = Span::new_extra("'hello\n", Config::default());
            let res = string(input);
            assert!(res.is_err(), "should fail on unclosed string with newline");

            if let Err(nom::Err::Failure(token_error)) = res {
                assert!(
                    matches!(
                        token_error.kind,
                        TokenErrorKind::Incomplete(IncompleteKind::UnclosedString { .. })
                    ),
                    "error should be for UnclosedString"
                );
            } else {
                panic!(
                    "expected TokenError Failure but got different error type: {:?}",
                    res
                );
            }
        }
    }
}
