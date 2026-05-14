/**
 * Converts Oneil identifiers to LaTeX math names.
 *
 * Each `_`-separated segment is looked up in `SYMBOL_MAP` first so Greek
 * letters and common math symbols become their LaTeX commands; everything else
 * passes through as math-italic text.  Segments after the first become
 * subscripts.
 *
 * These helpers are used both in the expression-to-LaTeX converter and
 * directly by components that need to render parameter names independently
 * of a full expression.
 */

// ── Symbol translation table ──────────────────────────────────────────────────

/**
 * Maps plain-text identifier parts to their LaTeX symbol equivalents.
 * Applied to every `_`-separated segment of an identifier.
 */
const SYMBOL_MAP: Record<string, string> = {
    // ── Lowercase Greek ───────────────────────────────────────────────────────
    alpha:      "\\alpha",
    beta:       "\\beta",
    gamma:      "\\gamma",
    delta:      "\\delta",
    epsilon:    "\\epsilon",
    varepsilon: "\\varepsilon",
    zeta:       "\\zeta",
    eta:        "\\eta",
    theta:      "\\theta",
    vartheta:   "\\vartheta",
    iota:       "\\iota",
    kappa:      "\\kappa",
    lambda:     "\\lambda",
    mu:         "\\mu",
    nu:         "\\nu",
    xi:         "\\xi",
    pi:         "\\pi",
    varpi:      "\\varpi",
    rho:        "\\rho",
    varrho:     "\\varrho",
    sigma:      "\\sigma",
    varsigma:   "\\varsigma",
    tau:        "\\tau",
    upsilon:    "\\upsilon",
    phi:        "\\phi",
    varphi:     "\\varphi",
    chi:        "\\chi",
    psi:        "\\psi",
    omega:      "\\omega",
    // ── Uppercase Greek ───────────────────────────────────────────────────────
    Gamma:   "\\Gamma",
    Delta:   "\\Delta",
    Theta:   "\\Theta",
    Lambda:  "\\Lambda",
    Xi:      "\\Xi",
    Pi:      "\\Pi",
    Sigma:   "\\Sigma",
    Upsilon: "\\Upsilon",
    Phi:     "\\Phi",
    Psi:     "\\Psi",
    Omega:   "\\Omega",
    // ── Common math symbols ───────────────────────────────────────────────────
    inf:   "\\infty",
    infty: "\\infty",
}

// Sort keys longest-first so that longer keys (e.g. "varepsilon") are tried
// before shorter ones that share a prefix (e.g. "epsilon") in the alternation.
const _SYMBOL_KEYS_BY_LENGTH = Object.keys(SYMBOL_MAP).sort(
    (a, b) => b.length - a.length,
)

// Pre-compiled regex that matches any SYMBOL_MAP key at a word boundary.
// The \b anchors ensure that a symbol name embedded inside a longer
// alphanumeric segment is never replaced — for example "pimax" must not
// yield "\\pimax" or "\\pi max".  This is already guaranteed by the
// _-split in `mathName`, but the explicit \b makes the contract visible.
const SYMBOL_RE = new RegExp(`\\b(${_SYMBOL_KEYS_BY_LENGTH.join("|")})\\b`)

function escapeIdent(s: string): string {
    return s.replace(/[#$%&_{}\\^~]/g, (c) => `\\${c}`)
}

/**
 * Converts a single identifier segment to its LaTeX form.
 * Symbol-table entries become their LaTeX commands; everything else is
 * passed through as math-italic text.
 *
 * Only an exact whole-segment match triggers substitution: a symbol name that
 * is a prefix or suffix of a longer segment (e.g. `"pimax"`, `"alphabetical"`)
 * is left untouched.
 */
export function partToLatex(part: string): string {
    const m = SYMBOL_RE.exec(part)
    if (m !== null && m[0] === part) return SYMBOL_MAP[m[1]]
    return escapeIdent(part)
}

/**
 * Converts an Oneil identifier to a LaTeX string.
 *
 * Each `_`-separated segment is looked up in `SYMBOL_MAP` first; if found it
 * becomes the corresponding LaTeX symbol, otherwise it renders in math italic.
 * Segments after the first become subscripts.
 *
 * @example
 * ```ts
 * mathName("m_pl")    // → "m_{pl}"
 * mathName("omega")   // → "\\omega"
 * mathName("theta_p") // → "\\theta_{p}"
 * mathName("m_omega") // → "m_{\\omega}"
 * mathName("g")       // → "g"
 * ```
 */
export function mathName(name: string): string {
    const parts = name.split("_")
    if (parts.length === 1) return partToLatex(name)
    const [head, ...tail] = parts
    return `${partToLatex(head)}_{${tail.map(partToLatex).join("\\,")}}`
}

/**
 * Like `mathName`, but appends a submodel reference alias as a
 * `\text{[alias]}` tag at the end of the subscript group.
 * When the parameter name has no underscores a new subscript is created for
 * the tag alone.
 *
 * @example
 * ```ts
 * mathNameWithRef("m_something", "submodel_a")
 * // → "m_{something\\text{[submodel\\_a]}}"
 * mathNameWithRef("g", "planet")
 * // → "g_{\\text{[planet]}}"
 * ```
 */
export function mathNameWithRef(paramName: string, refAlias: string): string {
    const safeAlias = refAlias.replace(/_/g, "\\_")
    const refTag = `\\text{[${safeAlias}]}`
    const parts = paramName.split("_")
    if (parts.length === 1) {
        return `${partToLatex(parts[0])}_{${refTag}}`
    }
    const [head, ...tail] = parts
    return `${partToLatex(head)}_{${tail.map(partToLatex).join("\\,")}${refTag}}`
}
