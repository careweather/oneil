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
; on the note-level captures is deliberate and load-bearing here: the
; default behavior (excluding named children's ranges from the parent's
; injected content) would carve `note_math` spans out of the Markdown
; document as gaps -- which sounds harmless, but for a `note_math` that
; sits mid-paragraph or mid-table-cell would hand Markdown a
; discontinuous document assembled from unrelated fragments (breaking
; e.g. a table whose header separator row is no longer adjacent to its
; header row). Including children instead keeps Markdown looking at
; *exactly* the same contiguous text it always did (math delimiters and
; all, which Markdown itself treats as inert) -- this and the
; `note_math` -> `latex` injection below are simply two independent,
; overlapping views of the same source range, exactly like a `<script>`
; tag being both part of an HTML document and its own JS document.
; `{{param:equation}}`/`{{param:value}}` interpolation placeholders are
; not (and structurally cannot be, without further grammar changes)
; highlighted specially; they're inert Markdown text, same as any other
; note prose.

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
