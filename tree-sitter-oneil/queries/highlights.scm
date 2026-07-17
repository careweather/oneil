; Highlighting queries for the Oneil modeling language.
;
; Capture names follow the conventions documented at
; https://tree-sitter.github.io/tree-sitter/3-syntax-highlighting.html#highlights,
; mirroring the scopes already used in `vscode/syntaxes/oneil.tmLanguage.json`
; where a reasonable equivalent exists.

; ---------------------------------------------------------------------------
; Generic fallbacks (kept first: more specific patterns below override these
; for the same node, per the usual "last match wins" highlight convention).
; ---------------------------------------------------------------------------

(identifier) @variable
(unit_identifier) @type
(label) @string.special

; ---------------------------------------------------------------------------
; Keywords
; ---------------------------------------------------------------------------

[
  "design"
  "import"
  "apply"
  "submodel"
  "reference"
  "section"
  "test"
] @keyword

"to" @keyword.operator
"as" @keyword.operator
"if" @keyword.conditional

[
  "not"
  "and"
  "or"
] @keyword.operator

(boolean) @constant.builtin.boolean

; ---------------------------------------------------------------------------
; Comments and notes
; ---------------------------------------------------------------------------

(comment) @comment
(single_line_note) @comment.documentation
(multi_line_note) @comment.documentation
; Overrides the `@comment.documentation` scope above for just the math
; span, for consumers that don't apply `queries/injections.scm`'s `latex`
; injection.
(note_math) @string.special

; ---------------------------------------------------------------------------
; Literals
; ---------------------------------------------------------------------------

(number) @number
(string) @string
(render_name) @string.special

; ---------------------------------------------------------------------------
; Punctuation and operators
; ---------------------------------------------------------------------------

[ "(" ")" "[" "]" "{" ] @punctuation.bracket
[ "," "." ] @punctuation.delimiter
[ ":" "=" ] @punctuation.special

(performance) @attribute
(trace_level) @attribute

[
  "+" "-" "--" "*" "/" "//" "%" "^" "?" "|"
  "<" "<=" ">" ">=" "==" "!="
] @operator

; ---------------------------------------------------------------------------
; Declarations
; ---------------------------------------------------------------------------

(design_target target: (identifier) @type)
(import path: (identifier) @module)

(apply file: (identifier) @module)
(apply_block file: (identifier) @module)
(model_path (identifier) @type)

(model_info name: (identifier) @type)
(model_info subcomponent: (identifier) @property)
(model_info alias: (identifier) @type)

(section label: (label) @markup.heading)
(parameter label: (label) @property)

(parameter name: (identifier) @variable)
(design_parameter name: (identifier) @variable)
(design_parameter instance: (identifier) @property)

; ---------------------------------------------------------------------------
; Expressions
; ---------------------------------------------------------------------------

(variable name: (identifier) @variable)
(variable reference: (identifier) @property)

(function_call function: (identifier) @function)
