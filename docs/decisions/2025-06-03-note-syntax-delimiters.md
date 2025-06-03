# Note Syntax Delimiters

## Status

Pending

## Context

In the Oneil language, we need a clear and unambiguous way to denote notes
within the code. Notes serve as documentation and explanatory text that should
be easily distinguishable from other language constructs. The main requirements
in order of priority are:

1. Easily readable by both humans and AI
2. Clear visual distinction from code and comments
3. Easily able to be typed

This decision supersedes the need for [inline whitespace
tokenization](2024-03-20-whitespace-tokenization.md), as the tilde-based syntax
removes the syntax that relies on inline whitespace.

## Decision

We propose to use tilde (`~`) based delimiters for notes:

1. Single-line notes:
   - Begin with a single tilde (`~`)
   - Continue until the end of the line
   - Example: `~ This is a single line note`

2. Multi-line notes:
   - Begin with three or more tildes (`~~~`)
   - End with the same number of tildes
   - Can contain any content except the closing delimiter
   - Example:
     ```
     ~~~
     This is a
     multi-line note
     ~~~
     ```

## Consequences

### Positive

1. **Unambiguous Parsing**: The tilde-based syntax is easy to parse since:
   - Single-line notes are clearly marked by a leading `~`
   - Multi-line notes have distinct start/end markers
   - No confusion with other language constructs (comments use `#`)

2. **Visual Distinction**: 
   - The tilde character is visually distinctive and not commonly used in
     mathematical or programming notation
   - Different from the hash (`#`) used for comments, making it clear these
     serve a different purpose

3. **Flexibility**:
   - Multi-line notes can contain any content (except the closing delimiter)
   - The syntax is simple enough to be easily remembered

### Negative

1. **Additional Syntax**: Introduces another special character (`~`) that users
   need to remember

2. **Unintuitive**: The tilde character could make the syntax feel less
   intuitive to new users.

## Alternatives Considered

1. **Whitespace-based Delimiting**:
   - Used by the current implementation
   - Requires less typing (no need for adding tildes)
   - Would have been more challenging to parse unambiguously
   - Could lead to formatting issues
   - Would make it harder to distinguish notes from other constructs

2. **Comment-based Notes**:
   - Could have used special comment markers (e.g., `##` or `###`)
   - Rejected to maintain clear distinction between comments and notes
   - Would make the purpose of different documentation constructs less clear

## Implementation Notes

The grammar has been updated to reflect this decision:
- Single-line notes are defined by the `SingleLineNote` rule
- Multi-line notes are defined by the `MultiLineNote` rule
- Both are captured under the general `Note` production 