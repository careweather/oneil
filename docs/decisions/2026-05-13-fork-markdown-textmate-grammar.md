# Fork text.html.markdown for Oneil Note Highlighting

## Status

Proposed (future work)

## Context

Oneil multi-line notes use `~~~` as both the opening and closing delimiter, and their
content is always uniformly indented in source (matching the parameter indentation level).
The content is Markdown with embedded LaTeX math (`$...$`, `$$...$$`) and Oneil-specific
parameter interpolation (`{{var:equation}}`, `{{var:value}}`).

VS Code's built-in `text.html.markdown` (CommonMark/GFM) grammar has two rules that are
fundamentally incompatible with embedding it directly inside Oneil note regions:

1. **`~~~` fenced code blocks** — the markdown grammar matches `~~~` as a fenced code
   block opener, conflicting with our own `~~~` note delimiter. Injection grammars were
   tried (`injectTo`, `injectionSelector`, `L:` priority) but couldn't reliably intercept
   the delimiter before the embedded grammar processed it. A `while`-based region was also
   tried, which works, but the fenced region is given to the child grammar as is, which includes indentation.

2. **4-space indented code blocks** — CommonMark treats any line indented by 4 or more
   spaces as an indented code block. Because note content is uniformly indented in source
   (e.g. 4 spaces), embedding `text.html.markdown` directly causes headings (`# …`), list
   items (`- …`), block quotes (`> …`) and other block constructs to be misidentified as
   code blocks rather than their intended elements. The Oneil parser uses a `dedent` pass
   to strip this common indent before processing, but the TextMate grammar has no
   equivalent transform.

The current workaround is a hand-written `#note-markdown-inline` repository in
`oneil.tmLanguage.json` that provides indentation-agnostic patterns for the most common
constructs: ATX headings, block quotes, ordered/unordered lists, bold, bold-italic, italic,
strikethrough, inline code, images, and links. LaTeX math is handled by `#note-math` and
Oneil interpolation by `#note-placeholder`. This covers the majority of real-world note
content but does not support fenced code blocks, nested structures, reference-style links,
or other grammar features that rely on the full CommonMark state machine.

## Decision

Fork `text.html.markdown` into `oneil/vscode/syntaxes/markdown-oneil.tmLanguage.json`
and make the following targeted modifications:

1. **Remove the `~~~` fenced code block rule** (the `begin: "~~~"` pattern in the fenced
   code fence section). The ` ``` ` fenced code block rule is unaffected and can remain.

2. **Remove or adjust the 4-space indented code block rule** so that uniformly-indented
   content is not classified as code. The simplest approach is to delete the
   `"match": "^( {4}|\\t)"` indented-code pattern entirely. A more precise approach would
   be to require the indent to be RELATIVE to the current note's base indent, but that
   requires dynamic capture information that TextMate grammars cannot express.

3. Replace `"include": "#note-markdown-inline"` in both note regions with
   `"include": "source.markdown.oneil"` (the new forked grammar's scope name), combined
   with `#note-placeholder` and `#note-math` as priority overrides listed first.

4. Update the `embeddedLanguages` map in `package.json` to associate the new scope name
   with `"markdown"` so that VS Code language services treat embedded content as markdown.

**Maintenance burden**: whenever VS Code ships an update to `text.html.markdown`, diff it
against our fork and pull in any non-conflicting improvements. The two deleted rules are
small and stable; the diff should typically be trivial.

## Consequences

**Easier:**
- Block-level markdown constructs (headings, lists, block quotes, tables, fenced code
  blocks using ` ``` `) highlight correctly inside notes regardless of indentation.
- The language spec and rendered output (which already dedents) are now consistent with
  the syntax highlighting.
- The hand-written `#note-markdown-inline` repository can be deleted, reducing maintenance.

**Harder:**
- VS Code grammar updates must be tracked and selectively merged into the fork. This is
  low-frequency (the base markdown grammar rarely changes) but requires attention.
- The fork must be stored in the repository and documented so contributors understand why
  it diverges from the standard grammar.
