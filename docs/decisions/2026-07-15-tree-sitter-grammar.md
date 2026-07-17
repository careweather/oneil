# Tree-sitter Grammar as a Permissive Superset

## Status

Accepted

## Context

`tree-sitter-oneil/grammar.js` provides incremental parsing for editor tooling (syntax
highlighting, folding, structural navigation, injected-language highlighting for notes and
render names) alongside the canonical EBNF (`docs/specs/grammar.ebnf`) and the authoritative
Rust recursive-descent parser (`src-rs/oneil_parser`). Tree-sitter's parsing model differs from
both in ways that force real design trade-offs:

- Tree-sitter compiles to a GLR parser over a **longest-match, non-backtracking lexer**. The
  Rust parser backtracks and commits to a specific rule/interpretation before lexing further;
  tree-sitter has no equivalent. Several constructs (labels immediately followed by other
  tokens, a dotted instance path, fenced multi-line notes) are lexically ambiguous under this
  model in ways they aren't for the Rust parser.
- The Rust parser tracks **file-level context** (`ModelPath::is_design_file()`,
  `require_design_header`/`allow_design_shorthand`) to decide whether `design_target`/shorthand
  `design_parameter` are legal at a given position. Tree-sitter has no such side channel.
- Tree-sitter has no built-in "note" concept. Note content (Markdown with embedded LaTeX math)
  and render names (raw LaTeX) are only useful to editor tooling if identifiable as *nodes* that
  can be handed off to `markdown`/`latex` injections (`queries/injections.scm`), not just
  characters the lexer swallows.

Rather than route around these individually, this ADR records the general decision: **the
tree-sitter grammar is intentionally a permissive superset of the reference language**, favoring
robustness and editor-tooling usefulness over rejecting invalid programs. Strict validation
remains the Rust parser's job.

## Decision

1. **Accept `design_target`/`design_parameter` anywhere a declaration is accepted**, not just at
   the top of a `.one` design file.
   - One grammar serves both `.on` and `.one` files.
   - A `.on` file using design-file-only syntax is accepted by tree-sitter, rejected by the Rust
     parser later — same as any other Rust-only validation.

2. **Resolve lexical ambiguities with GLR forking (`conflicts`) where more lookahead genuinely
   resolves them, and with explicit precedence/exclusion rules where it doesn't:**
   - `directory_path`/`model_path` segments: a `conflicts` entry; GLR forks and resolves once it
     sees whether a `/` or `.` follows.
   - A label's first word vs. a shorthand `design_parameter`'s name: also `conflicts`, resolved
     once `=` or `:`/limits appears.
   - A label's first word vs. a keyword, and a label continuation word vs. a fresh
     `identifier`-shaped declaration: not resolvable by forking alone (both stay "valid so far"
     indefinitely), so resolved lexically instead — tree-sitter's `word` mechanism (keywords win
     ties over identifiers/labels) plus a higher explicit `prec()` on continuation words. See the
     comment above `label` in `grammar.js` for the full reasoning, including why a few
     operator-leading characters and `.` are excluded from a label word's *first* character only.

3. **Treat newlines as significant**, matching `oneil_parser::token::structure::end_of_line`,
   rather than folding them into `extras` (the usual tree-sitter convention).
   - Making `\n` a blanket extra would reintroduce exactly the ambiguity (next declaration, or a
     label continuation word?) the reference grammar's line-oriented design avoids.
   - `_end_of_line` requires one or more newlines/comment lines, or literal end of file — the
     latter via a small external-scanner token (`_eof`), since a plain `token()` can't match
     "nothing, but only right here."

4. **Implement multi-line notes with an external scanner**, not a `token()` regex.
   - Tree-sitter's lexer is a longest-match DFA with no backtracking, so even a lazy repetition
     would greedily extend a fenced note to the *last* fence-like line in the file, not the
     first valid closing fence. Matching `oneil_parser::token::note`'s actual behavior needs the
     hand-written scan in `src/scanner.c`.

5. **Split note content into `note_math` child nodes plus opaque surrounding text**, and inject
   Markdown and LaTeX as two independent, overlapping layers (rather than one opaque note token,
   or separately-injected fragments):
   - A note's outer `markdown` injection captures the *entire* note (including `$.../$$...$$`
     delimiters) via `injection.include-children`, so Markdown always sees the same contiguous
     text it always did. This matters for block structures like tables — excluding `note_math`
     as a "gap" would hand Markdown a discontinuous document (e.g. a table row severed from its
     header).
   - `note_math` spans get their own, separate `latex` injection over the *same* bytes — like an
     HTML `<script>` tag being part of the HTML document and its own independent JS document.
   - `single_line_note`'s math (a plain `grammar.js` token) can't cross a line break, simply
     because `single_line_note` itself can't. `multi_line_note`'s math (`scan_math` in
     `src/scanner.c`) *can* span multiple lines, including a `$$...$$` block: nothing legitimate
     in math content looks like a tilde fence, so hitting a valid closing fence (or true EOF)
     before the closing delimiter unambiguously means the span is unterminated — no line-break
     restriction needed as a proxy for that. Either way, unterminated math is simply tolerated
     as ordinary text.
   - Consequence, documented in `src/scanner.c`: splitting note content into multiple tokens
     gives GLR error recovery an intermediate fallback point, so in one narrow case (unclosed
     note whose last content is valid math immediately followed by the file's final line break)
     it can silently accept the note as closed with no `(ERROR)` node. Accepted as a cosmetic
     editor-tooling gap — the Rust parser doesn't parse note content at all, so this can't affect
     what Rust considers valid, and well-formed notes are unaffected.

6. **Match `render_name` (`{...}`) with a bounded-depth nested-brace regex** (6 levels, via
   `nestedBraceContent`) rather than a true context-free match. Render names hold short LaTeX
   snippets (e.g. `{A_{\mathrm{s}}}`); a bounded-depth regex is a pragmatic stand-in given
   `token()` regexes can't recurse.

## Consequences

**Easier:**
- One grammar serves both `.on` and `.one` files, and both note-content forms, without
  threading file-kind/note-kind context through parsing rules the way the Rust parser does.
- Editor tooling gets real Markdown and LaTeX highlighting inside notes and render names via
  standard injections, rather than a hand-maintained approximation (contrast with
  `2026-05-13-fork-markdown-textmate-grammar.md`'s TextMate-side workaround).
- Test corpus files (`test/corpus/*.txt`) can focus on shape/structure rather than validity;
  invalid-but-structurally-parseable input is expected to parse into *something*.

**Harder:**
- The grammar accepts strictly more programs than the reference language. Anyone relying on it
  for validation (rather than editor tooling) will get false negatives — intentional, but should
  be called out wherever it's reused for a new purpose.
- Lexical tie-breaking rules (label vs. keyword vs. identifier, math vs. plain text) rely on
  tree-sitter-specific mechanisms (`word`, `prec`, `conflicts`) with no Rust-parser equivalent;
  changes to `oneil_parser`'s tokenization need an independently-reasoned update here, not a
  mechanical port.
- The external scanner (`src/scanner.c`) carries real control-flow complexity (lookahead that
  can't be un-consumed, multiple tokens cooperating to reconstruct one construct); changes to
  note syntax need scanner changes too, and regressions there (see the EOF/error-recovery
  caveat above) are easier to introduce than in a declarative rule.
