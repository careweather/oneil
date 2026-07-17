#include "tree_sitter/parser.h"

#include <stdbool.h>

// External scanner providing the tokens `grammar.js` can't express as plain
// regexes.
//
// - A multi-line note is split into four tokens (`_multi_line_note_open`,
//   `_multi_line_note_text`, `_multi_line_note_math`,
//   `_multi_line_note_close`) so `$...$`/`$$...$$` math spans become their
//   own node and can be injected as LaTeX, while the surrounding prose is
//   still injected as one contiguous Markdown document (see
//   `queries/injections.scm`).
// - Closing-fence detection needs a hand-written scanner: a regex `token()`
//   always prefers the longest match, so it would greedily extend a note to
//   the *last* fence-like line in the file instead of stopping at the
//   first, unlike the reference Rust parser (`oneil_parser::token::note`).
// - `scan_math` lets math spans cross line breaks: nothing legitimate in
//   math content looks like a tilde fence, so hitting one while searching
//   for the closing delimiter is unambiguous proof the span is
//   unterminated. The reference Rust parser doesn't parse note content at
//   all, so it has no opinion here either way.
// - `_eof`: a zero-width token matching only at literal end of file.
//   `_end_of_line` needs this as a fallback for a file whose final
//   declaration has no trailing line break (see `examples/mars.one`).
// - All scans are self-contained; no state is serialized between calls.
//
// Known limitation: splitting a note into several tokens gives GLR error
// recovery an intermediate fallback point, so in one narrow case (unclosed
// note whose last content is valid math immediately followed by the
// file's final line break) it can silently accept the note as closed with
// no `(ERROR)` node. Accepted as a cosmetic gap, not a soundness issue,
// since the Rust parser doesn't parse note content at all.

enum TokenType {
  MULTI_LINE_NOTE_OPEN,
  MULTI_LINE_NOTE_TEXT,
  MULTI_LINE_NOTE_MATH,
  MULTI_LINE_NOTE_CLOSE,
  EOF_TOKEN,
};

void *tree_sitter_oneil_external_scanner_create() { return NULL; }

void tree_sitter_oneil_external_scanner_destroy(void *payload) { (void)payload; }

unsigned tree_sitter_oneil_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}

void tree_sitter_oneil_external_scanner_deserialize(void *payload, const char *buffer,
                                                     unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

static inline bool is_inline_space(int32_t c) { return c == ' ' || c == '\t'; }
static inline bool is_eof(TSLexer *lexer) { return lexer->lookahead == 0; }

static void skip_inline_space(TSLexer *lexer, bool skip) {
  while (is_inline_space(lexer->lookahead)) {
    lexer->advance(lexer, skip);
  }
}

// Consumes `[ \t]*~~~+[ \t]*` starting at the current position.
// - Returns the number of tildes consumed (0 if not a fence).
// - Always advances through whatever it looks at, even on failure: there's
//   no backtracking, so a caller that decides it wasn't a real fence just
//   keeps the consumed characters as ordinary text instead.
static int consume_tilde_fence(TSLexer *lexer) {
  skip_inline_space(lexer, false);
  int tildes = 0;
  while (lexer->lookahead == '~') {
    lexer->advance(lexer, false);
    tildes++;
  }
  if (tildes >= 3) {
    skip_inline_space(lexer, false);
  }
  return tildes;
}

static bool at_line_end(TSLexer *lexer) {
  return lexer->lookahead == '\r' || lexer->lookahead == '\n' || is_eof(lexer);
}

static void consume_line_ending(TSLexer *lexer) {
  if (lexer->lookahead == '\r') {
    lexer->advance(lexer, false);
  }
  if (lexer->lookahead == '\n') {
    lexer->advance(lexer, false);
  }
}

// Outcome of `scan_math` below.
// - `MATH_UNTERMINATED`: lexer sits at a safe spot (EOF, or right after a
//   lone non-pairing `$`) -- fine to keep scanning as ordinary text.
// - `MATH_STOPPED_AT_FENCE`: lexer sits *past* fence characters that had
//   to be consumed to rule them out as math content. Must not be folded
//   into a text token (would swallow the note's own closing fence) -- the
//   caller must stop and return the pending text now, so a fresh call can
//   pick up the fence cleanly.
typedef enum { MATH_OK, MATH_UNTERMINATED, MATH_STOPPED_AT_FENCE } MathResult;

// Tries to match a `$...$` or `$$...$$` math span at the current position
// (`lexer->lookahead` is known to be `$`), with no further unescaped `$`
// in its content.
// - Can cross line breaks, unlike `NOTE_MATH` in `grammar.js` (which can't,
//   simply because `single_line_note` itself can't). A fence line hit
//   while searching for the closing delimiter unambiguously means the span
//   is unterminated.
// - On success (`MATH_OK`), advances through and marks the end at the
//   closing delimiter.
// - On failure, `mark_end` is left at the last position known *not* to be
//   part of a real closing fence -- see `MathResult` for what that means
//   for the caller. No backtracking is attempted (or possible).
static MathResult scan_math(TSLexer *lexer) {
  lexer->advance(lexer, false); // opening '$'
  bool block = false;
  if (lexer->lookahead == '$') {
    lexer->advance(lexer, false);
    block = true;
  }
  lexer->mark_end(lexer);
  while (lexer->lookahead != '$') {
    if (is_eof(lexer)) {
      return MATH_UNTERMINATED;
    }
    if (lexer->get_column(lexer) == 0) {
      uint32_t column_before = lexer->get_column(lexer);
      if (consume_tilde_fence(lexer) >= 3 && at_line_end(lexer)) {
        // A real fence wins over unterminated math. `mark_end` is already
        // right before it.
        return MATH_STOPPED_AT_FENCE;
      }
      if (lexer->get_column(lexer) != column_before) {
        lexer->mark_end(lexer); // commit the non-fence chars just inspected
        continue;
      }
      // Nothing consumed (didn't start with tilde/space) -- fall through.
    }
    if (at_line_end(lexer)) {
      consume_line_ending(lexer);
    } else {
      lexer->advance(lexer, false);
    }
    lexer->mark_end(lexer);
  }
  lexer->advance(lexer, false); // closing '$'
  if (block) {
    if (lexer->lookahead != '$') {
      // `mark_end` is already right before this stray, unpaired '$'.
      return MATH_UNTERMINATED;
    }
    lexer->advance(lexer, false);
  }
  lexer->mark_end(lexer);
  return MATH_OK;
}

// Scans the content of a multi-line note, one token at a time: a closing
// fence, a math span, or a run of text up to whichever comes first.
// `consumed_any` tracks whether this call has already committed text --
// once true, a fence/math span that would otherwise be returned directly
// instead just ends the text token right before it, left for a fresh call
// to pick up.
static bool scan_note_content(TSLexer *lexer, const bool *valid_symbols) {
  bool consumed_any = false;

  for (;;) {
    if (is_eof(lexer)) {
      // Unclosed note: no valid token here at all (matches the Rust
      // parser), so the whole scan fails, discarding any pending text.
      return false;
    }

    // Fence/math attempts below are gated on the relevant `valid_symbols`
    // entry before consuming anything: GLR can query this scanner more
    // than once at the same position with a narrower mask, to probe which
    // continuation applies. Trying unconditionally and succeeding while
    // that symbol wasn't even asked about would wrongly kill off a parse
    // branch that should have stayed alive (one expecting plain TEXT).
    if (lexer->get_column(lexer) == 0 && valid_symbols[MULTI_LINE_NOTE_CLOSE]) {
      uint32_t column_before = lexer->get_column(lexer);
      bool is_close = consume_tilde_fence(lexer) >= 3 && at_line_end(lexer);
      if (is_close) {
        if (!consumed_any) {
          lexer->mark_end(lexer);
          lexer->result_symbol = MULTI_LINE_NOTE_CLOSE;
          return true;
        }
        // `mark_end` already sits right before this fence.
        lexer->result_symbol = MULTI_LINE_NOTE_TEXT;
        return true;
      }
      if (lexer->get_column(lexer) != column_before) {
        // Not a real fence, but something was inspected (whitespace and/or
        // a few tildes) -- keep it as text and continue.
        consumed_any = true;
        lexer->mark_end(lexer);
        continue;
      }
    }

    if (lexer->lookahead == '$' && valid_symbols[MULTI_LINE_NOTE_MATH]) {
      if (consumed_any) {
        // `scan_math` would clobber this text token's `mark_end` on
        // success, with no way to move it back. Stop the text token here
        // instead, and let a fresh call (with `consumed_any` false) decide
        // whether this '$' starts valid math.
        lexer->result_symbol = MULTI_LINE_NOTE_TEXT;
        return true;
      }
      switch (scan_math(lexer)) {
        case MATH_OK:
          lexer->result_symbol = MULTI_LINE_NOTE_MATH;
          return true;
        case MATH_STOPPED_AT_FENCE:
          // Return the pending text now rather than falling through: the
          // loop below would extend past the fence (no "column 0" left to
          // notice it with) and silently swallow it. A fresh call, right
          // at the fence, picks it up cleanly.
          lexer->result_symbol = MULTI_LINE_NOTE_TEXT;
          return true;
        case MATH_UNTERMINATED:
          // No closing-fence hazard here -- safe to keep scanning as text.
          consumed_any = true;
          continue;
      }
    }

    if (!valid_symbols[MULTI_LINE_NOTE_TEXT]) {
      return false;
    }
    if (at_line_end(lexer)) {
      consume_line_ending(lexer);
    } else {
      lexer->advance(lexer, false);
    }
    consumed_any = true;
    lexer->mark_end(lexer);
  }
}

bool tree_sitter_oneil_external_scanner_scan(void *payload, TSLexer *lexer,
                                              const bool *valid_symbols) {
  (void)payload;

  // Checked first: `_eof` is zero-width, so a cheap no-op check anywhere
  // but true EOF. Order vs. `scan_note_content` doesn't matter at true
  // EOF -- both independently agree there's no valid content token there.
  if (valid_symbols[EOF_TOKEN] && is_eof(lexer)) {
    lexer->result_symbol = EOF_TOKEN;
    lexer->mark_end(lexer);
    return true;
  }

  if (valid_symbols[MULTI_LINE_NOTE_OPEN]) {
    if (consume_tilde_fence(lexer) < 3 || !at_line_end(lexer) || is_eof(lexer)) {
      // A fence with no line after it can't open a note: no room for
      // content or a closing fence.
      return false;
    }
    consume_line_ending(lexer);
    lexer->mark_end(lexer);
    lexer->result_symbol = MULTI_LINE_NOTE_OPEN;
    return true;
  }

  if (valid_symbols[MULTI_LINE_NOTE_TEXT] || valid_symbols[MULTI_LINE_NOTE_MATH] ||
      valid_symbols[MULTI_LINE_NOTE_CLOSE]) {
    return scan_note_content(lexer, valid_symbols);
  }

  return false;
}
