/*!
  Highlight.js v11.11.1 (git: 08cb242e7d)
  (c) 2006-2026 Josh Goebel <hello@joshgoebel.com> and other contributors
  License: BSD-3-Clause
 */

var hljs = function () { "use strict"; function e(t) { return t instanceof Map ? t.clear = t.delete = t.set = () => { throw Error("map is read-only") } : t instanceof Set && (t.add = t.clear = t.delete = () => { throw Error("set is read-only") }), Object.freeze(t), Object.getOwnPropertyNames(t).forEach((n => { const i = t[n], s = typeof i; "object" !== s && "function" !== s || Object.isFrozen(i) || e(i) })), t } class t { constructor(e) { void 0 === e.data && (e.data = {}), this.data = e.data, this.isMatchIgnored = !1 } ignoreMatch() { this.isMatchIgnored = !0 } } function n(e) { return e.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&#x27;") } function i(e, ...t) { const n = Object.create(null); for (const t in e) n[t] = e[t]; return t.forEach((e => { for (const t in e) n[t] = e[t] })), n } const s = e => !!e.scope; class r { constructor(e, t) { this.buffer = "", this.classPrefix = t.classPrefix, e.walk(this) } addText(e) { this.buffer += n(e) } openNode(e) { if (!s(e)) return; const t = ((e, { prefix: t }) => { if (e.startsWith("language:")) return e.replace("language:", "language-"); if (e.includes(".")) { const n = e.split("."); return [`${t}${n.shift()}`, ...n.map(((e, t) => `${e}${"_".repeat(t + 1)}`))].join(" ") } return `${t}${e}` })(e.scope, { prefix: this.classPrefix }); this.span(t) } closeNode(e) { s(e) && (this.buffer += "</span>") } value() { return this.buffer } span(e) { this.buffer += `<span class="${e}">` } } const o = (e = {}) => { const t = { children: [] }; return Object.assign(t, e), t }; class a { constructor() { this.rootNode = o(), this.stack = [this.rootNode] } get top() { return this.stack[this.stack.length - 1] } get root() { return this.rootNode } add(e) { this.top.children.push(e) } openNode(e) { const t = o({ scope: e }); this.add(t), this.stack.push(t) } closeNode() { if (this.stack.length > 1) return this.stack.pop() } closeAllNodes() { for (; this.closeNode();); } toJSON() { return JSON.stringify(this.rootNode, null, 4) } walk(e) { return this.constructor._walk(e, this.rootNode) } static _walk(e, t) { return "string" == typeof t ? e.addText(t) : t.children && (e.openNode(t), t.children.forEach((t => this._walk(e, t))), e.closeNode(t)), e } static _collapse(e) { "string" != typeof e && e.children && (e.children.every((e => "string" == typeof e)) ? e.children = [e.children.join("")] : e.children.forEach((e => { a._collapse(e) }))) } } class c extends a { constructor(e) { super(), this.options = e } addText(e) { "" !== e && this.add(e) } startScope(e) { this.openNode(e) } endScope() { this.closeNode() } __addSublanguage(e, t) { const n = e.root; t && (n.scope = "language:" + t), this.add(n) } toHTML() { return new r(this, this.options).value() } finalize() { return this.closeAllNodes(), !0 } } function l(e) { return e ? "string" == typeof e ? e : e.source : null } function g(e) { return h("(?=", e, ")") } function u(e) { return h("(?:", e, ")*") } function d(e) { return h("(?:", e, ")?") } function h(...e) { return e.map((e => l(e))).join("") } function f(...e) { const t = (e => { const t = e[e.length - 1]; return "object" == typeof t && t.constructor === Object ? (e.splice(e.length - 1, 1), t) : {} })(e); return "(" + (t.capture ? "" : "?:") + e.map((e => l(e))).join("|") + ")" } function p(e) { return RegExp(e.toString() + "|").exec("").length - 1 } const b = /\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./; function m(e, { joinWith: t }) { let n = 0; return e.map((e => { n += 1; const t = n; let i = l(e), s = ""; for (; i.length > 0;) { const e = b.exec(i); if (!e) { s += i; break } s += i.substring(0, e.index), i = i.substring(e.index + e[0].length), "\\" === e[0][0] && e[1] ? s += "\\" + (Number(e[1]) + t) : (s += e[0], "(" === e[0] && n++) } return s })).map((e => `(${e})`)).join(t) } const E = "[a-zA-Z]\\w*", x = "[a-zA-Z_]\\w*", y = "\\b\\d+(\\.\\d+)?", _ = "(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)", w = "\\b(0b[01]+)", O = { begin: "\\\\[\\s\\S]", relevance: 0 }, v = { scope: "string", begin: "'", end: "'", illegal: "\\n", contains: [O] }, k = { scope: "string", begin: '"', end: '"', illegal: "\\n", contains: [O] }, N = (e, t, n = {}) => { const s = i({ scope: "comment", begin: e, end: t, contains: [] }, n); s.contains.push({ scope: "doctag", begin: "[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)", end: /(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/, excludeBegin: !0, relevance: 0 }); const r = f("I", "a", "is", "so", "us", "to", "at", "if", "in", "it", "on", /[A-Za-z]+['](d|ve|re|ll|t|s|n)/, /[A-Za-z]+[-][a-z]+/, /[A-Za-z][a-z]{2,}/); return s.contains.push({ begin: h(/[ ]+/, "(", r, /[.]?[:]?([.][ ]|[ ])/, "){3}") }), s }, S = N("//", "$"), M = N("/\\*", "\\*/"), R = N("#", "$"); var j = Object.freeze({ __proto__: null, APOS_STRING_MODE: v, BACKSLASH_ESCAPE: O, BINARY_NUMBER_MODE: { scope: "number", begin: w, relevance: 0 }, BINARY_NUMBER_RE: w, COMMENT: N, C_BLOCK_COMMENT_MODE: M, C_LINE_COMMENT_MODE: S, C_NUMBER_MODE: { scope: "number", begin: _, relevance: 0 }, C_NUMBER_RE: _, END_SAME_AS_BEGIN: e => Object.assign(e, { "on:begin": (e, t) => { t.data._beginMatch = e[1] }, "on:end": (e, t) => { t.data._beginMatch !== e[1] && t.ignoreMatch() } }), HASH_COMMENT_MODE: R, IDENT_RE: E, MATCH_NOTHING_RE: /\b\B/, METHOD_GUARD: { begin: "\\.\\s*" + x, relevance: 0 }, NUMBER_MODE: { scope: "number", begin: y, relevance: 0 }, NUMBER_RE: y, PHRASAL_WORDS_MODE: { begin: /\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/ }, QUOTE_STRING_MODE: k, REGEXP_MODE: { scope: "regexp", begin: /\/(?=[^/\n]*\/)/, end: /\/[gimuy]*/, contains: [O, { begin: /\[/, end: /\]/, relevance: 0, contains: [O] }] }, RE_STARTERS_RE: "!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~", SHEBANG: (e = {}) => { const t = /^#![ ]*\//; return e.binary && (e.begin = h(t, /.*\b/, e.binary, /\b.*/)), i({ scope: "meta", begin: t, end: /$/, relevance: 0, "on:begin": (e, t) => { 0 !== e.index && t.ignoreMatch() } }, e) }, TITLE_MODE: { scope: "title", begin: E, relevance: 0 }, UNDERSCORE_IDENT_RE: x, UNDERSCORE_TITLE_MODE: { scope: "title", begin: x, relevance: 0 } }); function A(e, t) { "." === e.input[e.index - 1] && t.ignoreMatch() } function I(e, t) { void 0 !== e.className && (e.scope = e.className, delete e.className) } function T(e, t) { t && e.beginKeywords && (e.begin = "\\b(" + e.beginKeywords.split(" ").join("|") + ")(?!\\.)(?=\\b|\\s)", e.__beforeBegin = A, e.keywords = e.keywords || e.beginKeywords, delete e.beginKeywords, void 0 === e.relevance && (e.relevance = 0)) } function L(e, t) { Array.isArray(e.illegal) && (e.illegal = f(...e.illegal)) } function B(e, t) { if (e.match) { if (e.begin || e.end) throw Error("begin & end are not supported with match"); e.begin = e.match, delete e.match } } function P(e, t) { void 0 === e.relevance && (e.relevance = 1) } const D = (e, t) => { if (!e.beforeMatch) return; if (e.starts) throw Error("beforeMatch cannot be used with starts"); const n = Object.assign({}, e); Object.keys(e).forEach((t => { delete e[t] })), e.keywords = n.keywords, e.begin = h(n.beforeMatch, g(n.begin)), e.starts = { relevance: 0, contains: [Object.assign(n, { endsParent: !0 })] }, e.relevance = 0, delete n.beforeMatch }, H = ["of", "and", "for", "in", "not", "or", "if", "then", "parent", "list", "value"]; function C(e, t, n = "keyword") { const i = Object.create(null); return "string" == typeof e ? s(n, e.split(" ")) : Array.isArray(e) ? s(n, e) : Object.keys(e).forEach((n => { Object.assign(i, C(e[n], t, n)) })), i; function s(e, n) { t && (n = n.map((e => e.toLowerCase()))), n.forEach((t => { const n = t.split("|"); i[n[0]] = [e, $(n[0], n[1])] })) } } function $(e, t) { return t ? Number(t) : (e => H.includes(e.toLowerCase()))(e) ? 0 : 1 } const U = {}, z = e => { console.error(e) }, W = (e, ...t) => { console.log("WARN: " + e, ...t) }, X = (e, t) => { U[`${e}/${t}`] || (console.log(`Deprecated as of ${e}. ${t}`), U[`${e}/${t}`] = !0) }, G = Error(); function K(e, t, { key: n }) { let i = 0; const s = e[n], r = {}, o = {}; for (let e = 1; e <= t.length; e++)o[e + i] = s[e], r[e + i] = !0, i += p(t[e - 1]); e[n] = o, e[n]._emit = r, e[n]._multi = !0 } function F(e) { (e => { e.scope && "object" == typeof e.scope && null !== e.scope && (e.beginScope = e.scope, delete e.scope) })(e), "string" == typeof e.beginScope && (e.beginScope = { _wrap: e.beginScope }), "string" == typeof e.endScope && (e.endScope = { _wrap: e.endScope }), (e => { if (Array.isArray(e.begin)) { if (e.skip || e.excludeBegin || e.returnBegin) throw z("skip, excludeBegin, returnBegin not compatible with beginScope: {}"), G; if ("object" != typeof e.beginScope || null === e.beginScope) throw z("beginScope must be object"), G; K(e, e.begin, { key: "beginScope" }), e.begin = m(e.begin, { joinWith: "" }) } })(e), (e => { if (Array.isArray(e.end)) { if (e.skip || e.excludeEnd || e.returnEnd) throw z("skip, excludeEnd, returnEnd not compatible with endScope: {}"), G; if ("object" != typeof e.endScope || null === e.endScope) throw z("endScope must be object"), G; K(e, e.end, { key: "endScope" }), e.end = m(e.end, { joinWith: "" }) } })(e) } function Z(e) { function t(t, n) { return RegExp(l(t), "m" + (e.case_insensitive ? "i" : "") + (e.unicodeRegex ? "u" : "") + (n ? "g" : "")) } class n { constructor() { this.matchIndexes = {}, this.regexes = [], this.matchAt = 1, this.position = 0 } addRule(e, t) { t.position = this.position++, this.matchIndexes[this.matchAt] = t, this.regexes.push([t, e]), this.matchAt += p(e) + 1 } compile() { 0 === this.regexes.length && (this.exec = () => null); const e = this.regexes.map((e => e[1])); this.matcherRe = t(m(e, { joinWith: "|" }), !0), this.lastIndex = 0 } exec(e) { this.matcherRe.lastIndex = this.lastIndex; const t = this.matcherRe.exec(e); if (!t) return null; const n = t.findIndex(((e, t) => t > 0 && void 0 !== e)), i = this.matchIndexes[n]; return t.splice(0, n), Object.assign(t, i) } } class s { constructor() { this.rules = [], this.multiRegexes = [], this.count = 0, this.lastIndex = 0, this.regexIndex = 0 } getMatcher(e) { if (this.multiRegexes[e]) return this.multiRegexes[e]; const t = new n; return this.rules.slice(e).forEach((([e, n]) => t.addRule(e, n))), t.compile(), this.multiRegexes[e] = t, t } resumingScanAtSamePosition() { return 0 !== this.regexIndex } considerAll() { this.regexIndex = 0 } addRule(e, t) { this.rules.push([e, t]), "begin" === t.type && this.count++ } exec(e) { const t = this.getMatcher(this.regexIndex); t.lastIndex = this.lastIndex; let n = t.exec(e); if (this.resumingScanAtSamePosition()) if (n && n.index === this.lastIndex); else { const t = this.getMatcher(0); t.lastIndex = this.lastIndex + 1, n = t.exec(e) } return n && (this.regexIndex += n.position + 1, this.regexIndex === this.count && this.considerAll()), n } } if (e.compilerExtensions || (e.compilerExtensions = []), e.contains && e.contains.includes("self")) throw Error("ERR: contains `self` is not supported at the top-level of a language.  See documentation."); return e.classNameAliases = i(e.classNameAliases || {}), function n(r, o) { const a = r; if (r.isCompiled) return a;[I, B, F, D].forEach((e => e(r, o))), e.compilerExtensions.forEach((e => e(r, o))), r.__beforeBegin = null, [T, L, P].forEach((e => e(r, o))), r.isCompiled = !0; let c = null; return "object" == typeof r.keywords && r.keywords.$pattern && (r.keywords = Object.assign({}, r.keywords), c = r.keywords.$pattern, delete r.keywords.$pattern), c = c || /\w+/, r.keywords && (r.keywords = C(r.keywords, e.case_insensitive)), a.keywordPatternRe = t(c, !0), o && (r.begin || (r.begin = /\B|\b/), a.beginRe = t(a.begin), r.end || r.endsWithParent || (r.end = /\B|\b/), r.end && (a.endRe = t(a.end)), a.terminatorEnd = l(a.end) || "", r.endsWithParent && o.terminatorEnd && (a.terminatorEnd += (r.end ? "|" : "") + o.terminatorEnd)), r.illegal && (a.illegalRe = t(r.illegal)), r.contains || (r.contains = []), r.contains = [].concat(...r.contains.map((e => (e => (e.variants && !e.cachedVariants && (e.cachedVariants = e.variants.map((t => i(e, { variants: null }, t)))), e.cachedVariants ? e.cachedVariants : V(e) ? i(e, { starts: e.starts ? i(e.starts) : null }) : Object.isFrozen(e) ? i(e) : e))("self" === e ? r : e)))), r.contains.forEach((e => { n(e, a) })), r.starts && n(r.starts, o), a.matcher = (e => { const t = new s; return e.contains.forEach((e => t.addRule(e.begin, { rule: e, type: "begin" }))), e.terminatorEnd && t.addRule(e.terminatorEnd, { type: "end" }), e.illegal && t.addRule(e.illegal, { type: "illegal" }), t })(a), a }(e) } function V(e) { return !!e && (e.endsWithParent || V(e.starts)) } class q extends Error { constructor(e, t) { super(e), this.name = "HTMLInjectionError", this.html = t } } const J = n, Y = i, Q = Symbol("nomatch"), ee = n => { const i = Object.create(null), s = Object.create(null), r = []; let o = !0; const a = "Could not find the language '{}', did you forget to load/include a language module?", l = { disableAutodetect: !0, name: "Plain text", contains: [] }; let p = { ignoreUnescapedHTML: !1, throwUnescapedHTML: !1, noHighlightRe: /^(no-?highlight)$/i, languageDetectRe: /\blang(?:uage)?-([\w-]+)\b/i, classPrefix: "hljs-", cssSelector: "pre code", languages: null, __emitter: c }; function b(e) { return p.noHighlightRe.test(e) } function m(e, t, n) { let i = "", s = ""; "object" == typeof t ? (i = e, n = t.ignoreIllegals, s = t.language) : (X("10.7.0", "highlight(lang, code, ...args) has been deprecated."), X("10.7.0", "Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"), s = e, i = t), void 0 === n && (n = !0); const r = { code: i, language: s }; N("before:highlight", r); const o = r.result ? r.result : E(r.language, r.code, n); return o.code = r.code, N("after:highlight", o), o } function E(e, n, s, r) { const c = Object.create(null); function l() { if (!N.keywords) return void M.addText(R); let e = 0; N.keywordPatternRe.lastIndex = 0; let t = N.keywordPatternRe.exec(R), n = ""; for (; t;) { n += R.substring(e, t.index); const s = w.case_insensitive ? t[0].toLowerCase() : t[0], r = (i = s, N.keywords[i]); if (r) { const [e, i] = r; if (M.addText(n), n = "", c[s] = (c[s] || 0) + 1, c[s] <= 7 && (j += i), e.startsWith("_")) n += t[0]; else { const n = w.classNameAliases[e] || e; u(t[0], n) } } else n += t[0]; e = N.keywordPatternRe.lastIndex, t = N.keywordPatternRe.exec(R) } var i; n += R.substring(e), M.addText(n) } function g() { null != N.subLanguage ? (() => { if ("" === R) return; let e = null; if ("string" == typeof N.subLanguage) { if (!i[N.subLanguage]) return void M.addText(R); e = E(N.subLanguage, R, !0, S[N.subLanguage]), S[N.subLanguage] = e._top } else e = x(R, N.subLanguage.length ? N.subLanguage : null); N.relevance > 0 && (j += e.relevance), M.__addSublanguage(e._emitter, e.language) })() : l(), R = "" } function u(e, t) { "" !== e && (M.startScope(t), M.addText(e), M.endScope()) } function d(e, t) { let n = 1; const i = t.length - 1; for (; n <= i;) { if (!e._emit[n]) { n++; continue } const i = w.classNameAliases[e[n]] || e[n], s = t[n]; i ? u(s, i) : (R = s, l(), R = ""), n++ } } function h(e, t) { return e.scope && "string" == typeof e.scope && M.openNode(w.classNameAliases[e.scope] || e.scope), e.beginScope && (e.beginScope._wrap ? (u(R, w.classNameAliases[e.beginScope._wrap] || e.beginScope._wrap), R = "") : e.beginScope._multi && (d(e.beginScope, t), R = "")), N = Object.create(e, { parent: { value: N } }), N } function f(e, n, i) { let s = ((e, t) => { const n = e && e.exec(t); return n && 0 === n.index })(e.endRe, i); if (s) { if (e["on:end"]) { const i = new t(e); e["on:end"](n, i), i.isMatchIgnored && (s = !1) } if (s) { for (; e.endsParent && e.parent;)e = e.parent; return e } } if (e.endsWithParent) return f(e.parent, n, i) } function b(e) { return 0 === N.matcher.regexIndex ? (R += e[0], 1) : (T = !0, 0) } function m(e) { const t = e[0], i = n.substring(e.index), s = f(N, e, i); if (!s) return Q; const r = N; N.endScope && N.endScope._wrap ? (g(), u(t, N.endScope._wrap)) : N.endScope && N.endScope._multi ? (g(), d(N.endScope, e)) : r.skip ? R += t : (r.returnEnd || r.excludeEnd || (R += t), g(), r.excludeEnd && (R = t)); do { N.scope && M.closeNode(), N.skip || N.subLanguage || (j += N.relevance), N = N.parent } while (N !== s.parent); return s.starts && h(s.starts, e), r.returnEnd ? 0 : t.length } let y = {}; function _(i, r) { const a = r && r[0]; if (R += i, null == a) return g(), 0; if ("begin" === y.type && "end" === r.type && y.index === r.index && "" === a) { if (R += n.slice(r.index, r.index + 1), !o) { const t = Error(`0 width match regex (${e})`); throw t.languageName = e, t.badRule = y.rule, t } return 1 } if (y = r, "begin" === r.type) return (e => { const n = e[0], i = e.rule, s = new t(i), r = [i.__beforeBegin, i["on:begin"]]; for (const t of r) if (t && (t(e, s), s.isMatchIgnored)) return b(n); return i.skip ? R += n : (i.excludeBegin && (R += n), g(), i.returnBegin || i.excludeBegin || (R = n)), h(i, e), i.returnBegin ? 0 : n.length })(r); if ("illegal" === r.type && !s) { const e = Error('Illegal lexeme "' + a + '" for mode "' + (N.scope || "<unnamed>") + '"'); throw e.mode = N, e } if ("end" === r.type) { const e = m(r); if (e !== Q) return e } if ("illegal" === r.type && "" === a) return R += "\n", 1; if (I > 1e5 && I > 3 * r.index) throw Error("potential infinite loop, way more iterations than matches"); return R += a, a.length } const w = O(e); if (!w) throw z(a.replace("{}", e)), Error('Unknown language: "' + e + '"'); const v = Z(w); let k = "", N = r || v; const S = {}, M = new p.__emitter(p); (() => { const e = []; for (let t = N; t !== w; t = t.parent)t.scope && e.unshift(t.scope); e.forEach((e => M.openNode(e))) })(); let R = "", j = 0, A = 0, I = 0, T = !1; try { if (w.__emitTokens) w.__emitTokens(n, M); else { for (N.matcher.considerAll(); ;) { I++, T ? T = !1 : N.matcher.considerAll(), N.matcher.lastIndex = A; const e = N.matcher.exec(n); if (!e) break; const t = _(n.substring(A, e.index), e); A = e.index + t } _(n.substring(A)) } return M.finalize(), k = M.toHTML(), { language: e, value: k, relevance: j, illegal: !1, _emitter: M, _top: N } } catch (t) { if (t.message && t.message.includes("Illegal")) return { language: e, value: J(n), illegal: !0, relevance: 0, _illegalBy: { message: t.message, index: A, context: n.slice(A - 100, A + 100), mode: t.mode, resultSoFar: k }, _emitter: M }; if (o) return { language: e, value: J(n), illegal: !1, relevance: 0, errorRaised: t, _emitter: M, _top: N }; throw t } } function x(e, t) { t = t || p.languages || Object.keys(i); const n = (e => { const t = { value: J(e), illegal: !1, relevance: 0, _top: l, _emitter: new p.__emitter(p) }; return t._emitter.addText(e), t })(e), s = t.filter(O).filter(k).map((t => E(t, e, !1))); s.unshift(n); const r = s.sort(((e, t) => { if (e.relevance !== t.relevance) return t.relevance - e.relevance; if (e.language && t.language) { if (O(e.language).supersetOf === t.language) return 1; if (O(t.language).supersetOf === e.language) return -1 } return 0 })), [o, a] = r, c = o; return c.secondBest = a, c } function y(e) { let t = null; const n = (e => { let t = e.className + " "; t += e.parentNode ? e.parentNode.className : ""; const n = p.languageDetectRe.exec(t); if (n) { const t = O(n[1]); return t || (W(a.replace("{}", n[1])), W("Falling back to no-highlight mode for this block.", e)), t ? n[1] : "no-highlight" } return t.split(/\s+/).find((e => b(e) || O(e))) })(e); if (b(n)) return; if (N("before:highlightElement", { el: e, language: n }), e.dataset.highlighted) return void console.log("Element previously highlighted. To highlight again, first unset `dataset.highlighted`.", e); if (e.children.length > 0 && (p.ignoreUnescapedHTML || (console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."), console.warn("https://github.com/highlightjs/highlight.js/wiki/security"), console.warn("The element with unescaped HTML:"), console.warn(e)), p.throwUnescapedHTML)) throw new q("One of your code blocks includes unescaped HTML.", e.innerHTML); t = e; const i = t.textContent, r = n ? m(i, { language: n, ignoreIllegals: !0 }) : x(i); e.innerHTML = r.value, e.dataset.highlighted = "yes", ((e, t, n) => { const i = t && s[t] || n; e.classList.add("hljs"), e.classList.add("language-" + i) })(e, n, r.language), e.result = { language: r.language, re: r.relevance, relevance: r.relevance }, r.secondBest && (e.secondBest = { language: r.secondBest.language, relevance: r.secondBest.relevance }), N("after:highlightElement", { el: e, result: r, text: i }) } let _ = !1; function w() { if ("loading" === document.readyState) return _ || window.addEventListener("DOMContentLoaded", (() => { w() }), !1), void (_ = !0); document.querySelectorAll(p.cssSelector).forEach(y) } function O(e) { return e = (e || "").toLowerCase(), i[e] || i[s[e]] } function v(e, { languageName: t }) { "string" == typeof e && (e = [e]), e.forEach((e => { s[e.toLowerCase()] = t })) } function k(e) { const t = O(e); return t && !t.disableAutodetect } function N(e, t) { const n = e; r.forEach((e => { e[n] && e[n](t) })) } Object.assign(n, { highlight: m, highlightAuto: x, highlightAll: w, highlightElement: y, highlightBlock: e => (X("10.7.0", "highlightBlock will be removed entirely in v12.0"), X("10.7.0", "Please use highlightElement now."), y(e)), configure: e => { p = Y(p, e) }, initHighlighting: () => { w(), X("10.6.0", "initHighlighting() deprecated.  Use highlightAll() now.") }, initHighlightingOnLoad: () => { w(), X("10.6.0", "initHighlightingOnLoad() deprecated.  Use highlightAll() now.") }, registerLanguage: (e, t) => { let s = null; try { s = t(n) } catch (t) { if (z("Language definition for '{}' could not be registered.".replace("{}", e)), !o) throw t; z(t), s = l } s.name || (s.name = e), i[e] = s, s.rawDefinition = t.bind(null, n), s.aliases && v(s.aliases, { languageName: e }) }, unregisterLanguage: e => { delete i[e]; for (const t of Object.keys(s)) s[t] === e && delete s[t] }, listLanguages: () => Object.keys(i), getLanguage: O, registerAliases: v, autoDetection: k, inherit: Y, addPlugin: e => { (e => { e["before:highlightBlock"] && !e["before:highlightElement"] && (e["before:highlightElement"] = t => { e["before:highlightBlock"](Object.assign({ block: t.el }, t)) }), e["after:highlightBlock"] && !e["after:highlightElement"] && (e["after:highlightElement"] = t => { e["after:highlightBlock"](Object.assign({ block: t.el }, t)) }) })(e), r.push(e) }, removePlugin: e => { const t = r.indexOf(e); -1 !== t && r.splice(t, 1) } }), n.debugMode = () => { o = !1 }, n.safeMode = () => { o = !0 }, n.versionString = "11.11.1", n.regex = { concat: h, lookahead: g, either: f, optional: d, anyNumberOfTimes: u }; for (const t in j) "object" == typeof j[t] && e(j[t]); return Object.assign(n, j), n }, te = ee({}); return te.newInstance = () => ee({}), te }();

// languages retrieved from <https://github.com/highlightjs/highlight.js/tree/08cb242e7d4aee787114eb04cc7ab18314d82f92/src/languages>
hljs.registerLanguage("bash", function (hljs) { const regex = hljs.regex; const VAR = {}; const BRACED_VAR = { begin: /\$\{/, end: /\}/, contains: ["self", { begin: /:-/, contains: [VAR] }] }; Object.assign(VAR, { className: 'variable', variants: [{ begin: regex.concat(/\$[\w\d#@][\w\d_]*/, `(?![\\w\\d])(?![$])`) }, BRACED_VAR] }); const SUBST = { className: 'subst', begin: /\$\(/, end: /\)/, contains: [hljs.BACKSLASH_ESCAPE] }; const COMMENT = hljs.inherit(hljs.COMMENT(), { match: [/(^|\s)/, /#.*$/], scope: { 2: 'comment' } }); const HERE_DOC = { begin: /<<-?\s*(?=\w+)/, starts: { contains: [hljs.END_SAME_AS_BEGIN({ begin: /(\w+)/, end: /(\w+)/, className: 'string' })] } }; const QUOTE_STRING = { className: 'string', begin: /"/, end: /"/, contains: [hljs.BACKSLASH_ESCAPE, VAR, SUBST] }; SUBST.contains.push(QUOTE_STRING); const ESCAPED_QUOTE = { match: /\\"/ }; const APOS_STRING = { className: 'string', begin: /'/, end: /'/ }; const ESCAPED_APOS = { match: /\\'/ }; const ARITHMETIC = { begin: /\$?\(\(/, end: /\)\)/, contains: [{ begin: /\d+#[0-9a-f]+/, className: "number" }, hljs.NUMBER_MODE, VAR] }; const SH_LIKE_SHELLS = ["fish", "bash", "zsh", "sh", "csh", "ksh", "tcsh", "dash", "scsh",]; const KNOWN_SHEBANG = hljs.SHEBANG({ binary: `(${SH_LIKE_SHELLS.join("|")})`, relevance: 10 }); const FUNCTION = { className: 'function', begin: /\w[\w\d_]*\s*\(\s*\)\s*\{/, returnBegin: true, contains: [hljs.inherit(hljs.TITLE_MODE, { begin: /\w[\w\d_]*/ })], relevance: 0 }; const KEYWORDS = ["if", "then", "else", "elif", "fi", "time", "for", "while", "until", "in", "do", "done", "case", "esac", "coproc", "function", "select"]; const LITERALS = ["true", "false"]; const PATH_MODE = { match: /(\/[a-z._-]+)+/ }; const SHELL_BUILT_INS = ["break", "cd", "continue", "eval", "exec", "exit", "export", "getopts", "hash", "pwd", "readonly", "return", "shift", "test", "times", "trap", "umask", "unset"]; const BASH_BUILT_INS = ["alias", "bind", "builtin", "caller", "command", "declare", "echo", "enable", "help", "let", "local", "logout", "mapfile", "printf", "read", "readarray", "source", "sudo", "type", "typeset", "ulimit", "unalias"]; const ZSH_BUILT_INS = ["autoload", "bg", "bindkey", "bye", "cap", "chdir", "clone", "comparguments", "compcall", "compctl", "compdescribe", "compfiles", "compgroups", "compquote", "comptags", "comptry", "compvalues", "dirs", "disable", "disown", "echotc", "echoti", "emulate", "fc", "fg", "float", "functions", "getcap", "getln", "history", "integer", "jobs", "kill", "limit", "log", "noglob", "popd", "print", "pushd", "pushln", "rehash", "sched", "setcap", "setopt", "stat", "suspend", "ttyctl", "unfunction", "unhash", "unlimit", "unsetopt", "vared", "wait", "whence", "where", "which", "zcompile", "zformat", "zftp", "zle", "zmodload", "zparseopts", "zprof", "zpty", "zregexparse", "zsocket", "zstyle", "ztcp"]; const GNU_CORE_UTILS = ["chcon", "chgrp", "chown", "chmod", "cp", "dd", "df", "dir", "dircolors", "ln", "ls", "mkdir", "mkfifo", "mknod", "mktemp", "mv", "realpath", "rm", "rmdir", "shred", "sync", "touch", "truncate", "vdir", "b2sum", "base32", "base64", "cat", "cksum", "comm", "csplit", "cut", "expand", "fmt", "fold", "head", "join", "md5sum", "nl", "numfmt", "od", "paste", "ptx", "pr", "sha1sum", "sha224sum", "sha256sum", "sha384sum", "sha512sum", "shuf", "sort", "split", "sum", "tac", "tail", "tr", "tsort", "unexpand", "uniq", "wc", "arch", "basename", "chroot", "date", "dirname", "du", "echo", "env", "expr", "factor", "groups", "hostid", "id", "link", "logname", "nice", "nohup", "nproc", "pathchk", "pinky", "printenv", "printf", "pwd", "readlink", "runcon", "seq", "sleep", "stat", "stdbuf", "stty", "tee", "test", "timeout", "tty", "uname", "unlink", "uptime", "users", "who", "whoami", "yes"]; return { name: 'Bash', aliases: ['sh', 'zsh'], keywords: { $pattern: /\b[a-z][a-z0-9._-]+\b/, keyword: KEYWORDS, literal: LITERALS, built_in: [...SHELL_BUILT_INS, ...BASH_BUILT_INS, "set", "shopt", ...ZSH_BUILT_INS, ...GNU_CORE_UTILS] }, contains: [KNOWN_SHEBANG, hljs.SHEBANG(), FUNCTION, ARITHMETIC, COMMENT, HERE_DOC, PATH_MODE, QUOTE_STRING, ESCAPED_QUOTE, APOS_STRING, ESCAPED_APOS, VAR] }; });
hljs.registerLanguage("diff", function (hljs) { const regex = hljs.regex; return { name: 'Diff', aliases: ['patch'], contains: [{ className: 'meta', relevance: 10, match: regex.either(/^@@ +-\d+,\d+ +\+\d+,\d+ +@@/, /^\*\*\* +\d+,\d+ +\*\*\*\*$/, /^--- +\d+,\d+ +----$/) }, { className: 'comment', variants: [{ begin: regex.either(/Index: /, /^index/, /={3,}/, /^-{3}/, /^\*{3} /, /^\+{3}/, /^diff --git/), end: /$/ }, { match: /^\*{15}$/ }] }, { className: 'addition', begin: /^\+/, end: /$/ }, { className: 'deletion', begin: /^-/, end: /$/ }, { className: 'addition', begin: /^!/, end: /$/ }] }; });
hljs.registerLanguage("json", function (hljs) { const ATTRIBUTE = { className: 'attr', begin: /"(\\.|[^\\"\r\n])*"(?=\s*:)/, relevance: 1.01 }; const PUNCTUATION = { match: /[{}[\],:]/, className: "punctuation", relevance: 0 }; const LITERALS = ["true", "false", "null"]; const LITERALS_MODE = { scope: "literal", beginKeywords: LITERALS.join(" "), }; return { name: 'JSON', aliases: ['jsonc'], keywords: { literal: LITERALS, }, contains: [ATTRIBUTE, PUNCTUATION, hljs.QUOTE_STRING_MODE, LITERALS_MODE, hljs.C_NUMBER_MODE, hljs.C_LINE_COMMENT_MODE, hljs.C_BLOCK_COMMENT_MODE], illegal: '\\S' }; });
hljs.registerLanguage("latex", function (hljs) { const regex = hljs.regex; const KNOWN_CONTROL_WORDS = regex.either(...['(?:NeedsTeXFormat|RequirePackage|GetIdInfo)', 'Provides(?:Expl)?(?:Package|Class|File)', '(?:DeclareOption|ProcessOptions)', '(?:documentclass|usepackage|input|include)', 'makeat(?:letter|other)', 'ExplSyntax(?:On|Off)', '(?:new|renew|provide)?command', '(?:re)newenvironment', '(?:New|Renew|Provide|Declare)(?:Expandable)?DocumentCommand', '(?:New|Renew|Provide|Declare)DocumentEnvironment', '(?:(?:e|g|x)?def|let)', '(?:begin|end)', '(?:part|chapter|(?:sub){0,2}section|(?:sub)?paragraph)', 'caption', '(?:label|(?:eq|page|name)?ref|(?:paren|foot|super)?cite)', '(?:alpha|beta|[Gg]amma|[Dd]elta|(?:var)?epsilon|zeta|eta|[Tt]heta|vartheta)', '(?:iota|(?:var)?kappa|[Ll]ambda|mu|nu|[Xx]i|[Pp]i|varpi|(?:var)rho)', '(?:[Ss]igma|varsigma|tau|[Uu]psilon|[Pp]hi|varphi|chi|[Pp]si|[Oo]mega)', '(?:frac|sum|prod|lim|infty|times|sqrt|leq|geq|left|right|middle|[bB]igg?)', '(?:[lr]angle|q?quad|[lcvdi]?dots|d?dot|hat|tilde|bar)'].map(word => word + '(?![a-zA-Z@:_])')); const L3_REGEX = new RegExp(['(?:__)?[a-zA-Z]{2,}_[a-zA-Z](?:_?[a-zA-Z])+:[a-zA-Z]*', '[lgc]__?[a-zA-Z](?:_?[a-zA-Z])*_[a-zA-Z]{2,}', '[qs]__?[a-zA-Z](?:_?[a-zA-Z])+', 'use(?:_i)?:[a-zA-Z]*', '(?:else|fi|or):', '(?:if|cs|exp):w', '(?:hbox|vbox):n', '::[a-zA-Z]_unbraced', '::[a-zA-Z:]'].map(pattern => pattern + '(?![a-zA-Z:_])').join('|')); const L2_VARIANTS = [{ begin: /[a-zA-Z@]+/ }, { begin: /[^a-zA-Z@]?/ }]; const DOUBLE_CARET_VARIANTS = [{ begin: /\^{6}[0-9a-f]{6}/ }, { begin: /\^{5}[0-9a-f]{5}/ }, { begin: /\^{4}[0-9a-f]{4}/ }, { begin: /\^{3}[0-9a-f]{3}/ }, { begin: /\^{2}[0-9a-f]{2}/ }, { begin: /\^{2}[\u0000-\u007f]/ }]; const CONTROL_SEQUENCE = { className: 'keyword', begin: /\\/, relevance: 0, contains: [{ endsParent: true, begin: KNOWN_CONTROL_WORDS }, { endsParent: true, begin: L3_REGEX }, { endsParent: true, variants: DOUBLE_CARET_VARIANTS }, { endsParent: true, relevance: 0, variants: L2_VARIANTS }] }; const MACRO_PARAM = { className: 'params', relevance: 0, begin: /#+\d?/ }; const DOUBLE_CARET_CHAR = { variants: DOUBLE_CARET_VARIANTS }; const SPECIAL_CATCODE = { className: 'built_in', relevance: 0, begin: /[$&^_]/ }; const MAGIC_COMMENT = { className: 'meta', begin: /% ?!(T[eE]X|tex|BIB|bib)/, end: '$', relevance: 10 }; const COMMENT = hljs.COMMENT('%', '$', { relevance: 0 }); const EVERYTHING_BUT_VERBATIM = [CONTROL_SEQUENCE, MACRO_PARAM, DOUBLE_CARET_CHAR, SPECIAL_CATCODE, MAGIC_COMMENT, COMMENT]; const BRACE_GROUP_NO_VERBATIM = { begin: /\{/, end: /\}/, relevance: 0, contains: ['self', ...EVERYTHING_BUT_VERBATIM] }; const ARGUMENT_BRACES = hljs.inherit(BRACE_GROUP_NO_VERBATIM, { relevance: 0, endsParent: true, contains: [BRACE_GROUP_NO_VERBATIM, ...EVERYTHING_BUT_VERBATIM] }); const ARGUMENT_BRACKETS = { begin: /\[/, end: /\]/, endsParent: true, relevance: 0, contains: [BRACE_GROUP_NO_VERBATIM, ...EVERYTHING_BUT_VERBATIM] }; const SPACE_GOBBLER = { begin: /\s+/, relevance: 0 }; const ARGUMENT_M = [ARGUMENT_BRACES]; const ARGUMENT_O = [ARGUMENT_BRACKETS]; const ARGUMENT_AND_THEN = function (arg, starts_mode) { return { contains: [SPACE_GOBBLER], starts: { relevance: 0, contains: arg, starts: starts_mode } }; }; const CSNAME = function (csname, starts_mode) { return { begin: '\\\\' + csname + '(?![a-zA-Z@:_])', keywords: { $pattern: /\\[a-zA-Z]+/, keyword: '\\' + csname }, relevance: 0, contains: [SPACE_GOBBLER], starts: starts_mode }; }; const BEGIN_ENV = function (envname, starts_mode) { return hljs.inherit({ begin: '\\\\begin(?=[ \t]*(\\r?\\n[ \t]*)?\\{' + envname + '\\})', keywords: { $pattern: /\\[a-zA-Z]+/, keyword: '\\begin' }, relevance: 0, }, ARGUMENT_AND_THEN(ARGUMENT_M, starts_mode)); }; const VERBATIM_DELIMITED_EQUAL = (innerName = "string") => { return hljs.END_SAME_AS_BEGIN({ className: innerName, begin: /(.|\r?\n)/, end: /(.|\r?\n)/, excludeBegin: true, excludeEnd: true, endsParent: true }); }; const VERBATIM_DELIMITED_ENV = function (envname) { return { className: 'string', end: '(?=\\\\end\\{' + envname + '\\})' }; }; const VERBATIM_DELIMITED_BRACES = (innerName = "string") => { return { relevance: 0, begin: /\{/, starts: { endsParent: true, contains: [{ className: innerName, end: /(?=\})/, endsParent: true, contains: [{ begin: /\{/, end: /\}/, relevance: 0, contains: ["self"] }], }] } }; }; const VERBATIM = [...['verb', 'lstinline'].map(csname => CSNAME(csname, { contains: [VERBATIM_DELIMITED_EQUAL()] })), CSNAME('mint', ARGUMENT_AND_THEN(ARGUMENT_M, { contains: [VERBATIM_DELIMITED_EQUAL()] })), CSNAME('mintinline', ARGUMENT_AND_THEN(ARGUMENT_M, { contains: [VERBATIM_DELIMITED_BRACES(), VERBATIM_DELIMITED_EQUAL()] })), CSNAME('url', { contains: [VERBATIM_DELIMITED_BRACES("link"), VERBATIM_DELIMITED_BRACES("link")] }), CSNAME('hyperref', { contains: [VERBATIM_DELIMITED_BRACES("link")] }), CSNAME('href', ARGUMENT_AND_THEN(ARGUMENT_O, { contains: [VERBATIM_DELIMITED_BRACES("link")] })), ...[].concat(...['', '\\*'].map(suffix => [BEGIN_ENV('verbatim' + suffix, VERBATIM_DELIMITED_ENV('verbatim' + suffix)), BEGIN_ENV('filecontents' + suffix, ARGUMENT_AND_THEN(ARGUMENT_M, VERBATIM_DELIMITED_ENV('filecontents' + suffix))), ...['', 'B', 'L'].map(prefix => BEGIN_ENV(prefix + 'Verbatim' + suffix, ARGUMENT_AND_THEN(ARGUMENT_O, VERBATIM_DELIMITED_ENV(prefix + 'Verbatim' + suffix))))])), BEGIN_ENV('minted', ARGUMENT_AND_THEN(ARGUMENT_O, ARGUMENT_AND_THEN(ARGUMENT_M, VERBATIM_DELIMITED_ENV('minted')))),]; return { name: 'LaTeX', aliases: ['tex'], contains: [...VERBATIM, ...EVERYTHING_BUT_VERBATIM] }; });
hljs.registerLanguage("markdown", function (hljs) { const regex = hljs.regex; const INLINE_HTML = { begin: /<\/?[A-Za-z_]/, end: '>', subLanguage: 'xml', relevance: 0 }; const HORIZONTAL_RULE = { begin: '^[-\\*]{3,}', end: '$' }; const CODE = { className: 'code', variants: [{ begin: '(`{3,})[^`](.|\\n)*?\\1`*[ ]*' }, { begin: '(~{3,})[^~](.|\\n)*?\\1~*[ ]*' }, { begin: '```', end: '```+[ ]*$' }, { begin: '~~~', end: '~~~+[ ]*$' }, { begin: '`.+?`' }, { begin: '(?=^( {4}|\\t))', contains: [{ begin: '^( {4}|\\t)', end: '(\\n)$' }], relevance: 0 }] }; const LIST = { className: 'bullet', begin: '^[ \t]*([*+-]|(\\d+\\.))(?=\\s+)', end: '\\s+', excludeEnd: true }; const LINK_REFERENCE = { begin: /^\[[^\n]+\]:/, returnBegin: true, contains: [{ className: 'symbol', begin: /\[/, end: /\]/, excludeBegin: true, excludeEnd: true }, { className: 'link', begin: /:\s*/, end: /$/, excludeBegin: true }] }; const URL_SCHEME = /[A-Za-z][A-Za-z0-9+.-]*/; const LINK = { variants: [{ begin: /\[.+?\]\[.*?\]/, relevance: 0 }, { begin: /\[.+?\]\(((data|javascript|mailto):|(?:http|ftp)s?:\/\/).*?\)/, relevance: 2 }, { begin: regex.concat(/\[.+?\]\(/, URL_SCHEME, /:\/\/.*?\)/), relevance: 2 }, { begin: /\[.+?\]\([./?&#].*?\)/, relevance: 1 }, { begin: /\[.*?\]\(.*?\)/, relevance: 0 }], returnBegin: true, contains: [{ match: /\[(?=\])/ }, { className: 'string', relevance: 0, begin: '\\[', end: '\\]', excludeBegin: true, returnEnd: true }, { className: 'link', relevance: 0, begin: '\\]\\(', end: '\\)', excludeBegin: true, excludeEnd: true }, { className: 'symbol', relevance: 0, begin: '\\]\\[', end: '\\]', excludeBegin: true, excludeEnd: true }] }; const BOLD = { className: 'strong', contains: [], variants: [{ begin: /_{2}(?!\s)/, end: /_{2}/ }, { begin: /\*{2}(?!\s)/, end: /\*{2}/ }] }; const ITALIC = { className: 'emphasis', contains: [], variants: [{ begin: /\*(?![*\s])/, end: /\*/ }, { begin: /_(?![_\s])/, end: /_/, relevance: 0 }] }; const BOLD_WITHOUT_ITALIC = hljs.inherit(BOLD, { contains: [] }); const ITALIC_WITHOUT_BOLD = hljs.inherit(ITALIC, { contains: [] }); BOLD.contains.push(ITALIC_WITHOUT_BOLD); ITALIC.contains.push(BOLD_WITHOUT_ITALIC); let CONTAINABLE = [INLINE_HTML, LINK];[BOLD, ITALIC, BOLD_WITHOUT_ITALIC, ITALIC_WITHOUT_BOLD].forEach(m => { m.contains = m.contains.concat(CONTAINABLE); }); CONTAINABLE = CONTAINABLE.concat(BOLD, ITALIC); const HEADER = { className: 'section', variants: [{ begin: '^#{1,6}', end: '$', contains: CONTAINABLE }, { begin: '(?=^.+?\\n[=-]{2,}$)', contains: [{ begin: '^[=-]*$' }, { begin: '^', end: "\\n", contains: CONTAINABLE }] }] }; const BLOCKQUOTE = { className: 'quote', begin: '^>\\s+', contains: CONTAINABLE, end: '$' }; const ENTITY = { scope: 'literal', match: /&([a-zA-Z0-9]+|#[0-9]{1,7}|#[Xx][0-9a-fA-F]{1,6});/ }; return { name: 'Markdown', aliases: ['md', 'mkdown', 'mkd'], contains: [HEADER, INLINE_HTML, LIST, BOLD, ITALIC, BLOCKQUOTE, CODE, HORIZONTAL_RULE, LINK, LINK_REFERENCE, ENTITY] }; });
hljs.registerLanguage("plaintext", function (hljs) { return { name: 'Plain text', aliases: ['text', 'txt'], disableAutodetect: true }; });
hljs.registerLanguage("python-repl", function (hljs) { return { aliases: ['pycon'], contains: [{ className: 'meta.prompt', starts: { end: / |$/, starts: { end: '$', subLanguage: 'python' } }, variants: [{ begin: /^>>>(?=[ ]|$)/ }, { begin: /^\.\.\.(?=[ ]|$)/ }] }] }; });
hljs.registerLanguage("python", function (hljs) { const regex = hljs.regex; const IDENT_RE = /[\p{XID_Start}_]\p{XID_Continue}*/u; const RESERVED_WORDS = ['and', 'as', 'assert', 'async', 'await', 'break', 'case', 'class', 'continue', 'def', 'del', 'elif', 'else', 'except', 'finally', 'for', 'from', 'global', 'if', 'import', 'in', 'is', 'lambda', 'match', 'nonlocal|10', 'not', 'or', 'pass', 'raise', 'return', 'try', 'while', 'with', 'yield']; const BUILT_INS = ['__import__', 'abs', 'all', 'any', 'ascii', 'bin', 'bool', 'breakpoint', 'bytearray', 'bytes', 'callable', 'chr', 'classmethod', 'compile', 'complex', 'delattr', 'dict', 'dir', 'divmod', 'enumerate', 'eval', 'exec', 'filter', 'float', 'format', 'frozenset', 'getattr', 'globals', 'hasattr', 'hash', 'help', 'hex', 'id', 'input', 'int', 'isinstance', 'issubclass', 'iter', 'len', 'list', 'locals', 'map', 'max', 'memoryview', 'min', 'next', 'object', 'oct', 'open', 'ord', 'pow', 'print', 'property', 'range', 'repr', 'reversed', 'round', 'set', 'setattr', 'slice', 'sorted', 'staticmethod', 'str', 'sum', 'super', 'tuple', 'type', 'vars', 'zip']; const LITERALS = ['__debug__', 'Ellipsis', 'False', 'None', 'NotImplemented', 'True']; const TYPES = ["Any", "Callable", "Coroutine", "Dict", "List", "Literal", "Generic", "Optional", "Sequence", "Set", "Tuple", "Type", "Union"]; const KEYWORDS = { $pattern: /[A-Za-z]\w+|__\w+__/, keyword: RESERVED_WORDS, built_in: BUILT_INS, literal: LITERALS, type: TYPES }; const PROMPT = { className: 'meta', begin: /^(>>>|\.\.\.) / }; const SUBST = { className: 'subst', begin: /\{/, end: /\}/, keywords: KEYWORDS, illegal: /#/ }; const LITERAL_BRACKET = { begin: /\{\{/, relevance: 0 }; const STRING = { className: 'string', contains: [hljs.BACKSLASH_ESCAPE], variants: [{ begin: /([uU]|[bB]|[rR]|[bB][rR]|[rR][bB])?'''/, end: /'''/, contains: [hljs.BACKSLASH_ESCAPE, PROMPT], relevance: 10 }, { begin: /([uU]|[bB]|[rR]|[bB][rR]|[rR][bB])?"""/, end: /"""/, contains: [hljs.BACKSLASH_ESCAPE, PROMPT], relevance: 10 }, { begin: /([fF][rR]|[rR][fF]|[fF])'''/, end: /'''/, contains: [hljs.BACKSLASH_ESCAPE, PROMPT, LITERAL_BRACKET, SUBST] }, { begin: /([fF][rR]|[rR][fF]|[fF])"""/, end: /"""/, contains: [hljs.BACKSLASH_ESCAPE, PROMPT, LITERAL_BRACKET, SUBST] }, { begin: /([uU]|[rR])'/, end: /'/, relevance: 10 }, { begin: /([uU]|[rR])"/, end: /"/, relevance: 10 }, { begin: /([bB]|[bB][rR]|[rR][bB])'/, end: /'/ }, { begin: /([bB]|[bB][rR]|[rR][bB])"/, end: /"/ }, { begin: /([fF][rR]|[rR][fF]|[fF])'/, end: /'/, contains: [hljs.BACKSLASH_ESCAPE, LITERAL_BRACKET, SUBST] }, { begin: /([fF][rR]|[rR][fF]|[fF])"/, end: /"/, contains: [hljs.BACKSLASH_ESCAPE, LITERAL_BRACKET, SUBST] }, hljs.APOS_STRING_MODE, hljs.QUOTE_STRING_MODE] }; const digitpart = '[0-9](_?[0-9])*'; const pointfloat = `(\\b(${digitpart}))?\\.(${digitpart})|\\b(${digitpart})\\.`; const lookahead = `\\b|${RESERVED_WORDS.join('|')}`; const NUMBER = { className: 'number', relevance: 0, variants: [{ begin: `(\\b(${digitpart})|(${pointfloat}))[eE][+-]?(${digitpart})[jJ]?(?=${lookahead})` }, { begin: `(${pointfloat})[jJ]?` }, { begin: `\\b([1-9](_?[0-9])*|0+(_?0)*)[lLjJ]?(?=${lookahead})` }, { begin: `\\b0[bB](_?[01])+[lL]?(?=${lookahead})` }, { begin: `\\b0[oO](_?[0-7])+[lL]?(?=${lookahead})` }, { begin: `\\b0[xX](_?[0-9a-fA-F])+[lL]?(?=${lookahead})` }, { begin: `\\b(${digitpart})[jJ](?=${lookahead})` }] }; const COMMENT_TYPE = { className: "comment", begin: regex.lookahead(/# type:/), end: /$/, keywords: KEYWORDS, contains: [{ begin: /# type:/ }, { begin: /#/, end: /\b\B/, endsWithParent: true }] }; const PARAMS = { className: 'params', variants: [{ className: "", begin: /\(\s*\)/, skip: true }, { begin: /\(/, end: /\)/, excludeBegin: true, excludeEnd: true, keywords: KEYWORDS, contains: ['self', PROMPT, NUMBER, STRING, hljs.HASH_COMMENT_MODE] }] }; SUBST.contains = [STRING, NUMBER, PROMPT]; return { name: 'Python', aliases: ['py', 'gyp', 'ipython'], unicodeRegex: true, keywords: KEYWORDS, illegal: /(<\/|\?)|=>/, contains: [PROMPT, NUMBER, { scope: 'variable.language', match: /\bself\b/ }, { beginKeywords: "if", relevance: 0 }, { match: /\bor\b/, scope: "keyword" }, STRING, COMMENT_TYPE, hljs.HASH_COMMENT_MODE, { match: [/\bdef/, /\s+/, IDENT_RE,], scope: { 1: "keyword", 3: "title.function" }, contains: [PARAMS] }, { variants: [{ match: [/\bclass/, /\s+/, IDENT_RE, /\s*/, /\(\s*/, IDENT_RE, /\s*\)/], }, { match: [/\bclass/, /\s+/, IDENT_RE], }], scope: { 1: "keyword", 3: "title.class", 6: "title.class.inherited", } }, { className: 'meta', begin: /^[\t ]*@/, end: /(?=#)|$/, contains: [NUMBER, PARAMS, STRING] }] }; });
hljs.registerLanguage("rust", function (hljs) { const regex = hljs.regex; const RAW_IDENTIFIER = /(r#)?/; const UNDERSCORE_IDENT_RE = regex.concat(RAW_IDENTIFIER, hljs.UNDERSCORE_IDENT_RE); const IDENT_RE = regex.concat(RAW_IDENTIFIER, hljs.IDENT_RE); const FUNCTION_INVOKE = { className: "title.function.invoke", relevance: 0, begin: regex.concat(/\b/, /(?!let|for|while|if|else|match\b)/, IDENT_RE, regex.lookahead(/\s*\(/)) }; const NUMBER_SUFFIX = '([ui](8|16|32|64|128|size)|f(32|64))\?'; const KEYWORDS = ["abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate", "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "try", "type", "typeof", "union", "unsafe", "unsized", "use", "virtual", "where", "while", "yield"]; const LITERALS = ["true", "false", "Some", "None", "Ok", "Err"]; const BUILTINS = ['drop ', "Copy", "Send", "Sized", "Sync", "Drop", "Fn", "FnMut", "FnOnce", "ToOwned", "Clone", "Debug", "PartialEq", "PartialOrd", "Eq", "Ord", "AsRef", "AsMut", "Into", "From", "Default", "Iterator", "Extend", "IntoIterator", "DoubleEndedIterator", "ExactSizeIterator", "SliceConcatExt", "ToString", "assert!", "assert_eq!", "bitflags!", "bytes!", "cfg!", "col!", "concat!", "concat_idents!", "debug_assert!", "debug_assert_eq!", "env!", "eprintln!", "panic!", "file!", "format!", "format_args!", "include_bytes!", "include_str!", "line!", "local_data_key!", "module_path!", "option_env!", "print!", "println!", "select!", "stringify!", "try!", "unimplemented!", "unreachable!", "vec!", "write!", "writeln!", "macro_rules!", "assert_ne!", "debug_assert_ne!"]; const TYPES = ["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64", "str", "char", "bool", "Box", "Option", "Result", "String", "Vec"]; return { name: 'Rust', aliases: ['rs'], keywords: { $pattern: hljs.IDENT_RE + '!?', type: TYPES, keyword: KEYWORDS, literal: LITERALS, built_in: BUILTINS }, illegal: '</', contains: [hljs.C_LINE_COMMENT_MODE, hljs.COMMENT('/\\*', '\\*/', { contains: ['self'] }), hljs.inherit(hljs.QUOTE_STRING_MODE, { begin: /b?"/, illegal: null }), { className: 'symbol', begin: /'[a-zA-Z_][a-zA-Z0-9_]*(?!')/ }, { scope: 'string', variants: [{ begin: /b?r(#*)"(.|\n)*?"\1(?!#)/ }, { begin: /b?'/, end: /'/, contains: [{ scope: "char.escape", match: /\\('|\w|x\w{2}|u\w{4}|U\w{8})/ }] }] }, { className: 'number', variants: [{ begin: '\\b0b([01_]+)' + NUMBER_SUFFIX }, { begin: '\\b0o([0-7_]+)' + NUMBER_SUFFIX }, { begin: '\\b0x([A-Fa-f0-9_]+)' + NUMBER_SUFFIX }, { begin: '\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)' + NUMBER_SUFFIX }], relevance: 0 }, { begin: [/fn/, /\s+/, UNDERSCORE_IDENT_RE], className: { 1: "keyword", 3: "title.function" } }, { className: 'meta', begin: '#!?\\[', end: '\\]', contains: [{ className: 'string', begin: /"/, end: /"/, contains: [hljs.BACKSLASH_ESCAPE] }] }, { begin: [/let/, /\s+/, /(?:mut\s+)?/, UNDERSCORE_IDENT_RE], className: { 1: "keyword", 3: "keyword", 4: "variable" } }, { begin: [/for/, /\s+/, UNDERSCORE_IDENT_RE, /\s+/, /in/], className: { 1: "keyword", 3: "variable", 5: "keyword" } }, { begin: [/type/, /\s+/, UNDERSCORE_IDENT_RE], className: { 1: "keyword", 3: "title.class" } }, { begin: [/(?:trait|enum|struct|union|impl|for)/, /\s+/, UNDERSCORE_IDENT_RE], className: { 1: "keyword", 3: "title.class" } }, { begin: hljs.IDENT_RE + '::', keywords: { keyword: "Self", built_in: BUILTINS, type: TYPES } }, { className: "punctuation", begin: '->' }, FUNCTION_INVOKE] }; });
hljs.registerLanguage("shell", function (hljs) { return { name: 'Shell Session', aliases: ['console', 'shellsession'], contains: [{ className: 'meta.prompt', begin: /^\s{0,3}[/~\w\d[\]()@-]*[>%$#][ ]?/, starts: { end: /[^\\](?=\s*$)/, subLanguage: 'bash' } }] }; });
hljs.registerLanguage("plaintext", function (hljs) { return { name: 'Plain text', aliases: ['text', 'txt'], disableAutodetect: true }; });

// custom Oneil language (see docs/specs/grammar.ebnf)
//
// for highlight.js reference, see
// <https://highlightjs.readthedocs.io/en/latest/language-guide.html>
hljs.debugMode();
hljs.registerLanguage("oneil", function (hljs) {
  "use strict";
  const IDENT_RE = hljs.UNDERSCORE_IDENT_RE;
  const LABEL_RE = /[^()[\]{#~:= \t\n*$][^()[\]:=\n]*/;
  const END_OF_LINE_RE = /(?=#)|$/m; // the start of a comment is considered "end of line"

  // this is reused for both unit casting and parameter unit definition
  const UNIT_MODE = {
    scope: "type",
    begin: /:/,
    end: /\)|$/m,
    excludeBegin: true,
    excludeEnd: true,
  };

  const EXPR_MODE = {
    keywords: {
      keyword: ["and", "or", "not"],
      literal: ["true", "false", "inf"],
    },
    contains: [
      // strings
      hljs.APOS_STRING_MODE,

      // numbers
      hljs.C_NUMBER_MODE,

      // external parameter references - highlight the reference name
      {
        begin: [
          IDENT_RE,
          /\./,
          IDENT_RE,
        ],
        beginScope: {
          3: "title.class",
        },
      },

      // operators
      {
        scope: "operator",
        match: /--|\/\/|<=|>=|==|!=|[+\-*/%^?<>|]/,
      },

      // function calls
      {
        scope: "title.function.invoke",
        match: RegExp(IDENT_RE + "(?=\\s*\\()"),
      },

      // unit casting
      UNIT_MODE,
    ],
    endsWithParent: true,
  };

  // ── Note contents ───────────────────────────────────────────────────────────
  // Notes are markdown with embedded LaTeX math ($...$, $$...$$) and Oneil
  // {{var:equation}} / {{var:value}} interpolation placeholders.

  const NOTE_CONTAINS = [
    // display math: $$...$$  — subLanguage:"latex" highlights the content;
    // (?<!\\) prevents \$$ from closing the block.
    { scope: "formula", begin: /\$\$/, end: /(?<!\\)\$\$/, subLanguage: "latex" },
    // inline math: $..$ (not $$)
    { scope: "formula", begin: /\$(?!\$)/, end: /(?<!\\)\$/, subLanguage: "latex" },
    // {{var:equation}} and {{var:value}} placeholders
    { scope: "variable", match: /\{\{[\w]+:(?:equation|value)\}\}/ },
    // markdown bold+italic ***…***
    { scope: "strong", match: /\*{3}[^*\n]+\*{3}/ },
    // markdown bold **…** and __…__
    { scope: "strong", begin: /\*\*|__/, end: /\*\*|__/ },
    // markdown italic *…* (not part of ** or ***)
    { scope: "emphasis", match: /(?<!\*)\*(?!\*)[^*\n]+\*(?!\*)/ },
    // markdown strikethrough ~~…~~ (double tilde, not triple)
    { scope: "deletion", match: /~~(?!~)[^~\n]+~~/ },
    // ATX heading # ... ######
    { scope: "section", begin: /^\s*#{1,6}\s+/, end: /$/m },
    // blockquote > ...
    { scope: "quote", begin: /^\s*>+\s*/, end: /$/m },
    // unordered list item  - / * / +
    { scope: "bullet", match: /^\s*[-*+](?=\s)/ },
    // ordered list item  1. / 2.
    { scope: "bullet", match: /^\s*\d+\.(?=\s)/ },
    // image ![alt](url)
    { scope: "link", match: /!\[[^\]]*\]\([^)\s]+[^)]*\)/ },
    // link [text](url)
    { scope: "link", match: /\[[^\]]+\]\([^)\s]+[^)]*\)/ },
    // markdown inline code `…`
    { scope: "code", begin: /`+/, end: /`+/ },
    // TODO/FIXME/NOTE markers
    { scope: "doctag", match: /\b(?:TODO|FIXME|NOTE)\b/ },
  ];

  // ── Render name ─────────────────────────────────────────────────────────────
  // Optional LaTeX symbol immediately after the parameter colon, e.g. `{\hat{v}}`.
  // The lookahead confirms a balanced closing } on the same line so that
  // piecewise branches (whose { never closes on the same line) are not matched.
  const RENDER_NAME_MODE = {
    scope: "string",
    begin: /\s*\{(?=[^{}\n]*(?:\{[^{}\n]*\}[^{}\n]*)*\})/,
    end: /\}/,
    subLanguage: "latex",
  };

  return {
    name: "Oneil",
    aliases: [], // no aliases
    contains: [
      // single-line notes (~ prefix, not ~~~)
      {
        scope: "comment",
        begin: /^\s*~(?!~~)/m,
        end: /$/m,
        contains: NOTE_CONTAINS,
      },

      // multi-line notes (~~~ ... ~~~ block)
      {
        scope: "comment",
        begin: /^\s*~~~+\s*$/m,
        end: /^\s*~~~+\s*$/m,
        contains: NOTE_CONTAINS,
      },

      // hash comments
      hljs.HASH_COMMENT_MODE,

      // import declaration
      {
        begin: [
          /^\s*/m,
          /import/,
          /\s+/,
          IDENT_RE,
          RegExp(`\\s*(?=${END_OF_LINE_RE.source})`),
        ],
        beginScope: {
          2: "keyword",
          4: "variable",
        },
        end: END_OF_LINE_RE,
      },

      // submodel declaration
      {
        begin: [
          /^\s*/m,
          /submodel/,
          /\s+/,
          /[A-Za-z_][\w/]*/,
        ],
        beginScope: {
          2: "keyword",
          4: "title.class",
        },
        end: END_OF_LINE_RE,
        keywords: ["as"],
        contains: [
          // submodel foo as bar [inner, other as renamed]
          {
            begin: /\[/,
            end: /\]/,
            keywords: ["as"],
          }
        ],
      },

      // reference declaration
      {
        begin: [
          /^\s*/m,
          /reference/,
          /\s+/,
          /[A-Za-z_][\w/]*/,
        ],
        beginScope: {
          2: "keyword",
          4: "title.class",
        },
        end: END_OF_LINE_RE,
        keywords: ["as"],
      },

      // design declaration
      {
        begin: [
          /^\s*/m,
          /design/,
          /\s+/,
        ],
        beginScope: {
          2: "keyword",
        },
        end: END_OF_LINE_RE,
      },

      // apply declaration
      {
        begin: [
          /^\s*/m,
          /apply/,
          /\s+/,
          IDENT_RE,
        ],
        beginScope: {
          2: "keyword",
        },
        end: END_OF_LINE_RE,
        keywords: ["to"],
        contains: [
          {
            begin: /\[/,
            end: /\]/,
            keywords: ["to"],
          }
        ],
      },

      // section declaration
      {
        begin: [
          /^\s*/m,
          /section/,
          /\s+/,
          LABEL_RE,
          RegExp(`\\s*(?=${END_OF_LINE_RE.source})`),
        ],
        beginScope: {
          2: "keyword",
          4: "title",
        },
        end: END_OF_LINE_RE,
      },

      // test declaration
      {
        begin: [
          /^\s*/m,
          /test/,
          /\s*:/,
        ],
        beginScope: {
          2: "keyword",
        },
        end: END_OF_LINE_RE,
        contains: [
          EXPR_MODE,
        ]
      },

      // parameter definition (covers both model parameters and design overrides)
      {
        begin: [
          IDENT_RE,
          /\.?/,
          "(" + IDENT_RE + ")?",
          /\s*=/,
        ],
        beginScope: {
          1: "variable",
          3: "title.class",
        },
        end: END_OF_LINE_RE,
        keywords: ["if"],
        contains: [
          EXPR_MODE,
        ]
      },

      // render name: optional LaTeX symbol after the parameter colon, e.g. `{\hat{v}}`
      // must come before the piecewise branch so the balanced-brace lookahead can
      // distinguish render names from piecewise { branches.
      RENDER_NAME_MODE,

      // piecewise branch
      {
        begin: /\{/,
        end: END_OF_LINE_RE,
        keywords: ["if"],
        contains: [
          EXPR_MODE,
        ]
      },


      // parameter metadata (annotations, labels, limits)
      // 
      // this has to come last so that it doesn't override other modes
      {
        begin: [
          /^\s*(\$|\*\*?)?\s*/m, // includes `$`, `*`, `**` annotations
          LABEL_RE,
          /(?!\s*=)\s*/,         // reject bare `ident =` (shorthand param, not a label)
        ],
        beginScope: {
          2: "title",
        },
        end: /:\s*/m,
        contains: [
          // continuous limits
          {
            begin: /\(/,
            end: /\)\s*/,
            contains: [
              EXPR_MODE,
            ],
          },

          // discrete limits
          {
            begin: /\[/,
            end: /\]\s*/,
            contains: [
              EXPR_MODE,
            ],
          },
        ],
      },
    ],
  };
});

hljs.registerLanguage("oneil-eval-output", function (hljs) {
  // <expression or parameter> = <value> [:<unit>]  # <label>
  return {
    name: "oneil-eval-output",
    contains: [
      // no parameters message
      {
        scope: "comment",
        begin: /^\(No/m,
        end: /\)$/m,
      },

      // expression or parameter
      {
        scope: "variable",
        begin: /^/m,
        end: /(?= = )/,
        excludeBegin: true,
        excludeEnd: true,
      },

      // value
      {
        begin: /=/,
        end: /(?=[:#])|$/m,
        keywords: {
          literal: ["true", "false", "inf"],
        },
        contains: [
          hljs.C_NUMBER_MODE,
          hljs.APOS_STRING_MODE,
        ]
      },

      // unit
      {
        scope: "type",
        begin: /:/,
        end: /(?=#)|$/m,
        excludeBegin: true,
        excludeEnd: true,
      },

      // label
      {
        scope: "comment",
        begin: /#/,
        end: /$/m,
      },
    ]
  }
})
