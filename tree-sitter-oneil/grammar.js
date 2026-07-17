/**
 * @file Tree-sitter grammar for the Oneil modeling language
 * @author Careweather
 * @license MPL-2.0
 *
 * This grammar is derived from the canonical EBNF at `docs/specs/grammar.ebnf`
 * and cross-checked against the Rust recursive-descent parser in
 * `src-rs/oneil_parser`. It intentionally accepts a permissive *superset* of
 * the real language, favoring robustness for editor tooling (highlighting,
 * folding, structural navigation) over strict validation. See
 * `docs/decisions/2026-07-15-tree-sitter-grammar.md` for the rationale behind
 * the simplifications noted throughout this file.
 */

/* eslint-disable arrow-parens */
/* eslint-disable camelcase */
/* eslint-disable-next-line spaced-comment */

'use strict';

// Precedence levels for expressions, ascending from loosest to tightest
// binding, matching the table documented in `oneil_parser::expression` and
// the (highest-to-lowest) list in `docs/specs/grammar.ebnf`.
const PREC = {
  fallback: 1, // ?
  or: 2, // or
  and: 3, // and
  not: 4, // not (prefix)
  comparison: 5, // < > <= >= == != (chained, treated as left-assoc)
  minmax: 6, // |
  additive: 7, // + - --
  multiplicative: 8, // * / // %
  exponential: 9, // ^ (right-assoc)
  neg: 10, // unary -
};

/**
 * Builds a regex source string matching `{ ... }` content that may contain
 * up to `depth` levels of nested braces. Render names hold short LaTeX
 * snippets (e.g. `{\hat{v}}`, `{A_{\mathrm{s}}}`), so a bounded-depth regex
 * is a pragmatic stand-in for a true context-free (unbounded) match.
 */
function nestedBraceContent(depth) {
  let inner = '[^{}]*';
  for (let i = 0; i < depth; i++) {
    inner = `(?:[^{}]|\\{${inner}\\})*`;
  }
  return inner;
}

const RENDER_NAME = new RegExp(`\\{${nestedBraceContent(6)}\\}`);

// A note's content is Markdown with embedded LaTeX math (`$...$`/`$$...$$`;
// see `docs/specs/grammar.ebnf`).
// - `note_math` spans get their own node so `queries/injections.scm` can
//   inject `latex` into them, while the surrounding prose is still injected
//   as one contiguous `markdown` document via `injection.include-children`.
// - `multi_line_note`'s equivalent splitting happens in `src/scanner.c`
//   instead (fence detection there already needs a hand-written scanner).
//   Same "no unescaped `$` in content" rule, but this plain regex can never
//   cross a line break, simply because `single_line_note` itself can't.
const NOTE_MATH = /\$\$[^\n$]*\$\$|\$[^\n$]*\$/;

// Labels are context-sensitive "human" names that may contain spaces/tabs
// between words. See `oneil_parser::token::naming::label`.
//
// The label's *first* word is tokenized separately from the rest (rather
// than as one big `token()` covering the whole label), and is split further
// into "plain" (`$.identifier`-shaped) vs "punctuated" (starting with a
// character `identifier` disallows, e.g. a quote) cases. This matters for
// two distinct reasons:
//
// 1. Whenever a label's first word is spelled exactly like a reserved word
//    (`design`, `test`, ...) or like a plain identifier (the overwhelming
//    common case, e.g. `Mean` in `Mean radius: R = ...`), reusing
//    `$.identifier` means the lexer only ever produces *one* token for that
//    text. The resulting ambiguity -- is this identifier the start of a
//    `design_target`/`design_parameter`, or the first word of a `label`? --
//    then becomes a genuine parser-level (GLR) choice that later tokens
//    (`=` vs `:`/limits) resolve, and reserved words additionally get
//    resolved in the keyword's favor via tree-sitter's `word` mechanism.
//    Splitting the label's first word into its own `token()` (as before)
//    made this instead a *lexer*-level tie between two different token
//    types, which tree-sitter resolves with a fixed rule (declaration
//    order) rather than by looking ahead -- silently mis-parsing every
//    label whose first word happens to look like an identifier.
// 2. Every label word's *first* character additionally excludes the
//    leading character of every unit/binary-operator token that could
//    plausibly abut an identifier-like token with no separating whitespace
//    (`? | + / % ^ < > !`; not `-`, since `-` is never a unit operator and
//    labels commonly hyphenate words, e.g. `Delta-v budget: ...`), on top
//    of the reference grammar's own exclusions. Without this, a label
//    immediately following a now-complete value/unit with no separating
//    note or new declaration keyword (e.g. `g = 3.72 :m/s`, or even
//    `g = 3.72 :m` immediately followed by another label starting with
//    `/something`) is lexically indistinguishable, at the position right
//    after `m`, from the *start of the next declaration's label* -- and
//    tree-sitter's longest-match lexer would happily swallow `/s` as a
//    one-word label (or continuation word), pre-empting the shorter `/`
//    operator token that `unit_binary_expression` needs. This exclusion
//    only applies to the character *starting* a word: once a word is
//    underway, e.g. after the `-` in `Delta-v`, the rest of the reference
//    grammar's permissive character set applies. None of the real
//    `.on`/`.one` examples in this repo use these characters to *start* a
//    label word, so this loses nothing in practice.
// 3. For the same reason, `.` is excluded as a word-starting character
//    too: a `design_parameter` shorthand's dotted instance path
//    (`dv.c = ...`) tokenizes its `.` and the following identifier as two
//    separate tokens, and a label continuation word starting with `.`
//    directly abutting the previous word (e.g. the `.c` in `dv.c`) would
//    otherwise be a *longer* single-token match that the lexer's
//    maximal-munch rule would always prefer over those two shorter
//    tokens, incorrectly forcing the label interpretation even when the
//    surrounding declaration can only be a shorthand `design_parameter`.
//    Excluding `.` here just means such input tokenizes identically to
//    the instance-path case, leaving the choice to the parser (GLR),
//    which is exactly what happens for any other identifier-shaped first
//    word (see point 1) -- the label fork simply fails later (no `:`
//    before the mandatory delimiter) and is discarded.
//
// The reference (recursive-descent, backtracking) parser never faces
// either problem because it commits to a specific rule (and, within it, to
// "still parsing this value") before ever trying to lex a label.
const IDENT_CHAR = 'A-Za-z0-9_';
const LABEL_CONTINUE_EXCLUDE = '()\\[\\]#~:=\\t\\n ';
const LABEL_WORD_OPERATOR_EXCLUDE = '?|+/%^<>!.';
// Permissive: allowed anywhere in a word except as the word's first
// character.
const LABEL_CONTINUE_CHAR = `[^${LABEL_CONTINUE_EXCLUDE}]`;
// Restricted: required for the first character of *any* label word
// (first or subsequent).
const LABEL_WORD_START_CHAR = `[^${LABEL_CONTINUE_EXCLUDE}${LABEL_WORD_OPERATOR_EXCLUDE}]`;
// A first word starting with a character `identifier` disallows (so it
// can't be captured by reusing `$.identifier`): a hyphenated or otherwise
// identifier-adjacent first word (e.g. `Delta-v`) instead decomposes into
// `identifier` (`Delta`) followed by an other-word (`-v`), since words
// don't require separating whitespace (see `label` below).
const LABEL_PUNCTUATED_FIRST_WORD = new RegExp(
  `[^${LABEL_CONTINUE_EXCLUDE}${LABEL_WORD_OPERATOR_EXCLUDE}{*\\$${IDENT_CHAR}]${LABEL_CONTINUE_CHAR}*`,
);
const LABEL_OTHER_WORD = new RegExp(`${LABEL_WORD_START_CHAR}${LABEL_CONTINUE_CHAR}*`);

const IDENTIFIER = /[A-Za-z_][A-Za-z0-9_]*/;

// Gives a reserved word slightly higher lexical precedence than `label`'s
// first word (see above) and, via the `word` conflict mechanism, than
// `identifier`, so that e.g. `design` or `or` wins a length tie against a
// label/identifier that happens to spell exactly that word.
function keyword(word) {
  return token(prec(1, word));
}

const NUMBER = /[+-]?(?:[0-9]+(?:\.[0-9]+)?|\.[0-9]+)(?:[eE][+-]?[0-9]+)?/;
// No word-boundary assertion: the `regex` crate backing tree-sitter's lexer
// has no look-around. This is safe because `number` and `identifier` are
// both valid alternatives wherever a literal is expected, and tree-sitter's
// lexer prefers the longer match, so `infinity`/`influx` still lex as a
// single identifier rather than `inf` + a trailing identifier.
const INF_LITERAL = /[+-]?inf/;

module.exports = grammar({
  name: 'oneil',

  word: ($) => $.identifier,

  externals: ($) => [
    $._multi_line_note_open,
    $._multi_line_note_text,
    $._multi_line_note_math,
    $._multi_line_note_close,
    $._eof,
  ],

  // Only inline whitespace is insignificant. Unlike most grammars,
  // newlines are *not* extras here: the reference grammar (see
  // `oneil_parser::token::structure::end_of_line`) treats line breaks as a
  // required declaration/statement terminator (`EndOfLine` throughout
  // `docs/specs/grammar.ebnf`), and collapsing that structure away by
  // making `\n` a blanket extra reintroduces exactly the ambiguities the
  // reference parser's line-oriented design was built to avoid -- e.g.
  // whether a bare identifier right after a `label`'s words continues the
  // label or starts the next declaration is only decidable by knowing
  // whether a line break came first. See `_end_of_line` below.
  extras: ($) => [/[ \t]/],

  conflicts: ($) => [
    // Whether an upcoming identifier starts another `directory_path`
    // segment or is the following field (a target/model name) can only be
    // known once we see whether a `/` follows it, which is beyond LALR(1)
    // lookahead; let the GLR runtime fork and resolve it.
    [$.directory_path],
    // Same shift/reduce ambiguity as `directory_path`, but for the
    // dot-separated segments of a `model_path`.
    [$.model_path],
    // An identifier that could equally be the start of a `design_parameter`
    // (shorthand) or the first word of a `parameter`'s `label` (full form)
    // is only disambiguated once we see whether `=` or `:`/limits follows.
    [$.design_parameter, $._label_first_word],
    // After a `piecewise_part`, an upcoming `_end_of_line` could either
    // continue the piecewise value (if the following line starts another
    // `{ ... if ...` part) or terminate the whole declaration -- only
    // decidable by looking past the line break for a `{`, beyond LALR(1)
    // lookahead.
    [$.piecewise_value],
  ],

  rules: {
    // `Model = [ EndOfLine ], [ Note ], { Declaration } { Section }`.
    source_file: ($) =>
      seq(
        optional($._end_of_line),
        optional($.note),
        repeat($._declaration),
        repeat($.section),
      ),

    // ---------------------------------------------------------------------
    // Line structure
    // ---------------------------------------------------------------------

    // A required declaration/statement terminator: one or more line breaks
    // and/or comment-only lines (each comment already consumes its own
    // trailing line break -- or nothing, at true end of file -- so a bare
    // `$.comment` is a complete "line" on its own here), or, if there is no
    // trailing line break at all, literal end of file (`$._eof`, an
    // external, zero-width token; see `src/scanner.c`). Mirrors
    // `oneil_parser::token::structure::end_of_line` exactly.
    _end_of_line: ($) => choice(repeat1(choice($._newline, $.comment)), $._eof),
    _newline: () => /\r?\n/,
    comment: () => token(/#[^\n]*\r?\n?/),

    // ---------------------------------------------------------------------
    // Notes
    // ---------------------------------------------------------------------

    // A note is always followed by its own terminator: this both ends the
    // note's own line and absorbs any further blank/comment-only lines
    // before the next declaration, matching how the reference parser's
    // `note` token folds a full `end_of_line` into its own trailing
    // whitespace span.
    note: ($) => seq(choice($.single_line_note, $.multi_line_note), $._end_of_line),

    single_line_note: ($) =>
      seq('~', repeat(choice($.note_math, $._stray_dollar, $._note_text))),
    // Fallback for a `$` that isn't valid math (e.g. "Cost: $5/kg") -- just
    // ordinary content, not an error. Lower precedence than `note_math` is
    // a tie-breaker only in principle: the two can never match the same
    // span.
    _stray_dollar: () => token(prec(-1, '$')),
    _note_text: () => token(/[^\n$]+/),
    note_math: () => token(prec(1, NOTE_MATH)),

    // Composed from four external tokens (`src/scanner.c`): opening fence,
    // a repeated mix of math/text, closing fence. The closing fence token
    // stops right at the fence (doesn't consume its line break), matching
    // `single_line_note` so `$._end_of_line` handles both uniformly.
    multi_line_note: ($) =>
      seq(
        $._multi_line_note_open,
        repeat(choice(alias($._multi_line_note_math, $.note_math), $._multi_line_note_text)),
        $._multi_line_note_close,
      ),

    // ---------------------------------------------------------------------
    // Top level: sections & declarations
    // ---------------------------------------------------------------------

    section: ($) =>
      seq(
        keyword('section'),
        field('label', $.label),
        $._end_of_line,
        optional($.note),
        repeat($._declaration),
      ),

    // NOTE: Unlike the reference grammar, `design_target` and the shorthand
    // `design_parameter` form are accepted anywhere a declaration is
    // accepted (top level or inside a section), rather than only at the top
    // of a `.one` design file. This keeps a single grammar for both `.on`
    // and `.one` files; see the ADR for rationale.
    _declaration: ($) =>
      choice(
        $.design_target,
        $.import,
        $.apply,
        $.submodel,
        $.test,
        $.parameter,
        $.design_parameter,
      ),

    design_target: ($) =>
      seq(
        keyword('design'),
        optional($.directory_path),
        field('target', $.identifier),
        $._end_of_line,
      ),

    import: ($) => seq(keyword('import'), field('path', $.identifier), $._end_of_line),

    // -------------------------------------------------------------------
    // Apply
    // -------------------------------------------------------------------

    apply: ($) => seq(keyword('apply'), $._apply_body, $._end_of_line),

    _apply_body: ($) =>
      seq(
        optional($.directory_path),
        field('file', $.identifier),
        keyword('to'),
        field('target', $.model_path),
        optional($.apply_block),
      ),

    // Nested applies are permissively separated: a trailing comma, a line
    // break, both, or neither, matching `oneil_parser::declaration`'s
    // `nested_apply_item` (comma-or-end-of-line, then an independent
    // optional end-of-line).
    apply_block: ($) =>
      seq(
        '[',
        optional($._end_of_line),
        repeat(seq($._apply_body, optional(','), optional($._end_of_line))),
        ']',
      ),

    model_path: ($) => seq($.identifier, repeat(seq('.', $.identifier))),

    // -------------------------------------------------------------------
    // Submodel / reference
    // -------------------------------------------------------------------

    submodel: ($) =>
      seq(
        field('kind', choice(keyword('submodel'), keyword('reference'))),
        optional($.directory_path),
        field('model', $.model_info),
        optional($.submodel_list),
        $._end_of_line,
      ),

    directory_path: ($) => repeat1(seq($._directory_name, '/')),

    _directory_name: ($) => choice($.identifier, '..', '.'),

    model_info: ($) =>
      seq(
        field('name', $.identifier),
        repeat(seq('.', field('subcomponent', $.identifier))),
        optional(seq(keyword('as'), field('alias', $.identifier))),
      ),

    submodel_list: ($) =>
      seq(
        '[',
        optional($._end_of_line),
        optional(
          seq(
            $.model_info,
            repeat(seq(',', optional($._end_of_line), $.model_info)),
            optional(','),
            optional($._end_of_line),
          ),
        ),
        ']',
      ),

    // -------------------------------------------------------------------
    // Test
    // -------------------------------------------------------------------

    test: ($) =>
      seq(
        optional(field('trace', $.trace_level)),
        keyword('test'),
        ':',
        field('condition', $._expression),
        $._end_of_line,
        optional($.note),
      ),

    // -------------------------------------------------------------------
    // Parameters
    // -------------------------------------------------------------------

    performance: () => '$',
    trace_level: () => choice('**', '*'),

    // Full form: `[$] [*|**] Label [Limits] : [RenderName] name = value`
    parameter: ($) =>
      seq(
        optional(field('performance', $.performance)),
        optional(field('trace', $.trace_level)),
        field('label', $.label),
        optional(field('limits', $.limits)),
        ':',
        optional(field('render_name', $.render_name)),
        field('name', $.identifier),
        '=',
        field('value', $._parameter_value),
        $._end_of_line,
        optional($.note),
      ),

    // Shorthand form (design bodies only, but accepted anywhere for
    // simplicity): `[$] [*|**] name[.instance] [Limits] = value`
    design_parameter: ($) =>
      seq(
        optional(field('performance', $.performance)),
        optional(field('trace', $.trace_level)),
        field('name', $.identifier),
        optional(seq('.', field('instance', $.identifier))),
        optional(field('limits', $.limits)),
        '=',
        field('value', $._parameter_value),
        $._end_of_line,
        optional($.note),
      ),

    limits: ($) => choice($.continuous_limits, $.discrete_limits),

    continuous_limits: ($) =>
      seq(
        '(',
        field('min', $._expression),
        ',',
        field('max', $._expression),
        ')',
      ),

    discrete_limits: ($) =>
      seq('[', $._expression, repeat(seq(',', $._expression)), ']'),

    render_name: () => token(RENDER_NAME),

    _parameter_value: ($) => choice($.simple_value, $.piecewise_value),

    simple_value: ($) =>
      seq(
        field('expr', $._expression),
        optional(seq(':', field('unit', $._unit_expression))),
      ),

    // Pieces are one-per-line: `PiecewiseExpr = PiecewisePart, [":", UnitExpr],
    // { EndOfLine, PiecewisePart }`.
    piecewise_value: ($) =>
      seq(
        $.piecewise_part,
        optional(seq(':', field('unit', $._unit_expression))),
        repeat(seq($._end_of_line, $.piecewise_part)),
      ),

    // NOTE: faithful to the reference grammar, a piecewise part has no
    // closing brace: `{ expr if condition`.
    piecewise_part: ($) =>
      seq(
        '{',
        field('expr', $._expression),
        keyword('if'),
        field('condition', $._expression),
      ),

    // ---------------------------------------------------------------------
    // Expressions
    // ---------------------------------------------------------------------

    _expression: ($) =>
      choice(
        $.number,
        $.string,
        $.boolean,
        $.function_call,
        $.unit_cast,
        $.variable,
        $.parenthesized_expression,
        $.unary_expression,
        $.binary_expression,
      ),

    variable: ($) =>
      seq(
        field('name', $.identifier),
        optional(seq('.', field('reference', $.identifier))),
      ),

    function_call: ($) =>
      seq(
        field('function', $.identifier),
        '(',
        optional(seq($._expression, repeat(seq(',', $._expression)))),
        ')',
      ),

    unit_cast: ($) =>
      seq(
        '(',
        field('expr', $._expression),
        ':',
        field('unit', $._unit_expression),
        ')',
      ),

    parenthesized_expression: ($) => seq('(', $._expression, ')'),

    unary_expression: ($) =>
      choice(
        prec(PREC.not, seq(field('operator', keyword('not')), field('operand', $._expression))),
        prec(PREC.neg, seq(field('operator', '-'), field('operand', $._expression))),
      ),

    binary_expression: ($) =>
      choice(
        ...[
          [PREC.fallback, '?'],
          [PREC.or, keyword('or')],
          [PREC.and, keyword('and')],
          [PREC.minmax, '|'],
        ].map(([precedence, operator]) =>
          prec.left(
            precedence,
            seq(
              field('left', $._expression),
              field('operator', operator),
              field('right', $._expression),
            ),
          ),
        ),
        prec.left(
          PREC.comparison,
          seq(
            field('left', $._expression),
            field('operator', choice('<', '<=', '>', '>=', '==', '!=')),
            field('right', $._expression),
          ),
        ),
        prec.left(
          PREC.additive,
          seq(
            field('left', $._expression),
            field('operator', choice('+', '-', '--')),
            field('right', $._expression),
          ),
        ),
        prec.left(
          PREC.multiplicative,
          seq(
            field('left', $._expression),
            field('operator', choice('*', '/', '//', '%')),
            field('right', $._expression),
          ),
        ),
        prec.right(
          PREC.exponential,
          seq(
            field('left', $._expression),
            field('operator', '^'),
            field('right', $._expression),
          ),
        ),
      ),

    number: () => token(choice(NUMBER, INF_LITERAL)),
    string: () => token(/'[^'\n]*'/),
    boolean: () => choice(keyword('true'), keyword('false')),

    // ---------------------------------------------------------------------
    // Units
    // ---------------------------------------------------------------------

    _unit_expression: ($) =>
      choice(
        $.unit,
        $.unit_one,
        $.unit_binary_expression,
        $.parenthesized_unit_expression,
      ),

    unit: ($) =>
      seq(
        field('name', $.unit_identifier),
        optional(seq('^', field('exponent', $.number))),
      ),

    // A bare literal `1` denoting "unitless". No trailing-digit exclusion is
    // needed: nothing else in a unit expression can start with a digit, so
    // `12` simply fails to parse as a unit (correctly, since it isn't one).
    unit_one: () => '1',

    // Higher precedence than `simple_value`/`piecewise_value` (which are
    // left at the default) so that, after `... : <unit>`, a following `*`
    // always extends the unit expression rather than being treated as the
    // *next* declaration's trace-level marker. `m * s` as a unit is common;
    // a trace-level marker immediately abutting a unit with no separating
    // note/blank-line boundary is not.
    unit_binary_expression: ($) =>
      prec.left(
        1,
        seq(
          field('left', $._unit_expression),
          field('operator', choice('*', '/')),
          field('right', $._unit_expression),
        ),
      ),

    parenthesized_unit_expression: ($) => seq('(', $._unit_expression, ')'),

    unit_identifier: () =>
      token(choice(seq(IDENTIFIER, optional('$')), '$', '%')),

    // ---------------------------------------------------------------------
    // Lexical primitives
    // ---------------------------------------------------------------------

    identifier: () => IDENTIFIER,

    // No explicit inter-word separator: unlike most of this grammar (where
    // inline whitespace is uniformly an extra), using an explicit
    // `/[ \t]+/` token here would make the lexer commit to "there's
    // another label word coming" the moment it sees *any* whitespace,
    // before it can tell whether that whitespace is actually just
    // formatting before the next, unrelated token (e.g. the `=` of a
    // `design_parameter`) -- pre-empting the correct interpretation. Plain
    // extras avoid that. Because only inline whitespace (not `\n`) is an
    // extra (see `extras` above), a label still can't accidentally span a
    // line break: `_label_other_word` simply has no way to be reached once
    // a line break is due, since that position instead requires
    // `_end_of_line`.
    label: ($) => seq($._label_first_word, repeat($._label_other_word)),
    _label_first_word: ($) => choice($.identifier, token(LABEL_PUNCTUATED_FIRST_WORD)),
    // Higher precedence than `identifier`'s default (0): once inside a
    // label, an identifier-shaped *continuation* word (e.g. `Budget` in
    // `section Mass Budget`) should always keep extending the label rather
    // than being lexed as a plain `identifier` -- which, at this parser
    // position, would otherwise tie with `_label_other_word` (both match
    // the same text with the same length) and is only "valid" here because
    // a new declaration (e.g. a shorthand `design_parameter`) can also
    // start with a bare identifier. See the module comment above `label`
    // for why this ambiguity can't be resolved by GLR forking the way the
    // *first* word's identifier-vs-keyword/label choice is.
    _label_other_word: () => token(prec(1, LABEL_OTHER_WORD)),
  },
});
