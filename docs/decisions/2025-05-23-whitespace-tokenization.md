# Tokenize Whitespace as Explicit Tokens

## Status

Accepted

## Context

The Oneil programming language is whitespace-sensitive, particularly in its handling of notes. Notes are a key feature of the language that require precise whitespace handling to determine their structure and relationships. Traditional lexer implementations often discard whitespace during tokenization, but this would lose critical semantic information needed for proper note parsing.

## Decision

We will explicitly tokenize whitespace as first-class tokens during the lexical analysis phase, rather than discarding them. Whitespace will be categorized into two distinct types:

1. Inline Whitespace
   - Consists of spaces and tabs
   - Adjacent inline whitespace characters are grouped into a single token
   - Example: "    \t  " becomes one inline whitespace token
   - Used for horizontal spacing and indentation

2. Line Breaks
   - Consists of newline sequences (`\n` or `\r\n`)
   - Each line break is treated as a separate token
   - Critical for note structure and vertical layout

Implementation details:
1. The lexer will emit these two distinct token types:
   - `InlineWhitespace` - for grouped spaces and tabs
   - `LineBreak` - for newline sequences

2. These whitespace tokens will be preserved in the token stream and made available to the parser.

3. The parser will use these whitespace tokens to:
   - Determine note indentation levels using `InlineWhitespace` tokens
   - Track line boundaries using `LineBreak` tokens
   - Maintain source formatting information
   - Ensure proper syntax validation where whitespace is significant

## Consequences

### Positive

1. Preserves all necessary information for handling whitespace-sensitive note structures
2. Makes whitespace handling explicit and easier to reason about in the parser
3. Enables better error reporting related to whitespace issues
4. Maintains the ability to reconstruct the original source format if needed
5. Simplifies the implementation of note-related features that depend on indentation
6. Reduces token stream size by grouping adjacent inline whitespace
7. Clear distinction between horizontal and vertical whitespace makes parsing more intuitive

### Negative

1. Additional memory usage to store whitespace information
2. More complex token handling in the parser since it needs to process whitespace tokens
3. Need for clear documentation about whitespace token handling for maintainers
4. Grouping logic for inline whitespace adds minor complexity to the lexer 