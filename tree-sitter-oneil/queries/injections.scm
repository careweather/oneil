; Language injections for the Oneil modeling language.
;
; Per `docs/specs/grammar.ebnf`: "Note content is Markdown with embedded
; LaTeX math ($...$ and $$...$$)" and a `RenderName` is a raw LaTeX snippet
; (e.g. `{\hat{v}}`). Unlike the hand-rolled Markdown patterns in
; `vscode/syntaxes/oneil.tmLanguage.json` (which has to avoid TextMate's
; regex-based `text.html.markdown` include entirely, since that grammar's
; own `~~~`-fenced-code-block detection collides with Oneil's `~~~` note
; delimiters), it's safe to inject the real `markdown` grammar here: these
; captures are precise node ranges with the Oneil-specific delimiters
; already trimmed off by `#offset!`, so the injected text never contains a
; bare `~~~` line for `markdown` to misinterpret as its own fence.
;
; A note's `$...$`/`$$...$$` math spans are their own `note_math` child
; nodes (see `grammar.js`/`src/scanner.c`), captured below as a *second*,
; independent `latex` injection layered on top of the same bytes the
; `markdown` injection above already covers. `injection.include-children`
; is load-bearing here: without it, tree-sitter would carve each `note_math`
; span out of the parent's injected range as a gap, and the remaining
; fragments are handed to Markdown as *independent, non-spliced documents* --
; a table's header row and its data rows can end up in entirely separate
; Markdown parses and never form a table at all. Including children keeps
; Markdown looking at one contiguous region (math delimiters and all). The
; `$`/`$$` delimiters are inert in CommonMark; the only content that can
; disrupt Markdown inside math spans is a bare `|` (table column separator),
; which is a pre-existing Markdown table limitation -- use `\|` or `\mid`.
; Known limitation: note content with 4+ spaces of leading indentation is
; interpreted by Markdown as a code block. Indentation is the recommended
; style inside Oneil sections, so this affects real files. Acceptable for
; now since the VSCode TextMate highlighter is the primary tooling path and
; tree-sitter is supplementary.
; `{{param:equation}}`/`{{param:value}}` interpolation placeholders are
; inert Markdown text and not highlighted specially.

((single_line_note) @injection.content
  (#set! injection.language "markdown")
  (#set! injection.include-children)
  ; Strip the leading `~`.
  (#offset! @injection.content 0 1 0 0))

((multi_line_note) @injection.content
  (#set! injection.language "markdown")
  (#set! injection.include-children)
  ; Strip the opening `~~~` fence's whole line and the closing fence's
  ; whole line, keeping only the lines in between.
  (#offset! @injection.content 1 0 -1 999999))

; `$$...$$` (block) math spans strip two `$` from each side; `$...$`
; (inline) spans strip one. `note_math` is a single node type covering
; both forms (see `grammar.js`), so the two cases are told apart here by
; matching on the captured text itself.
((note_math) @injection.content
  (#match? @injection.content "^\\$\\$")
  (#set! injection.language "latex")
  (#offset! @injection.content 0 2 0 -2))

((note_math) @injection.content
  (#match? @injection.content "^\\$[^$]")
  (#set! injection.language "latex")
  (#offset! @injection.content 0 1 0 -1))

; `RenderName` (`{...}`) holds a raw LaTeX snippet for the parameter's
; rendered symbol, e.g. `{A_{\mathrm{s}}}` -> `A_{\mathrm{s}}`.
((render_name) @injection.content
  (#set! injection.language "latex")
  ; Strip the outer braces.
  (#offset! @injection.content 0 1 0 -1))
