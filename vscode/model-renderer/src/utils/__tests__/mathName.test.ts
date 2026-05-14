import { describe, it, expect } from "vitest"
import { mathName, mathNameWithRef, partToLatex } from "../mathName"

describe("mathName", () => {
    it.each([
        ["g",       "g"],
        ["omega",   "\\omega"],
        ["pi",      "\\pi"],
        ["inf",     "\\infty"],
        ["m_pl",    "m_{pl}"],
        ["theta_p", "\\theta_{p}"],
        ["m_omega", "m_{\\omega}"],
        ["a_b_c",   "a_{b\\,c}"],
        ["Sigma",   "\\Sigma"],
        // varepsilon / varphi / etc. must win over the shorter embedded key
        ["varepsilon",    "\\varepsilon"],
        ["varepsilon_s",  "\\varepsilon_{s}"],
    ])("mathName(%s) → %s", (input, expected) => {
        expect(mathName(input)).toBe(expected)
    })
})

describe("mathName – word boundary: symbol names must not match inside longer segments", () => {
    it.each([
        // Symbol name as a prefix of a longer segment
        ["pimax",        "pimax"],
        ["alphabetical", "alphabetical"],
        ["sigmafield",   "sigmafield"],
        ["omegastar",    "omegastar"],
        ["tauon",        "tauon"],
        // Symbol name as a suffix of a longer segment
        ["maxpi",        "maxpi"],
        ["vsigma",       "vsigma"],
        // Symbol name embedded in the middle
        ["apib",         "apib"],
    ])("partToLatex(%s) stays %s", (input, expected) => {
        expect(partToLatex(input)).toBe(expected)
    })
})

describe("mathNameWithRef", () => {
    it.each([
        ["g",           "planet",    "g_{\\text{[planet]}}"],
        ["m_something", "submodel_a","m_{something\\text{[submodel\\_a]}}"],
        ["thrust",      "engine",    "thrust_{\\text{[engine]}}"],
        ["omega",       "ref",       "\\omega_{\\text{[ref]}}"],
    ])("mathNameWithRef(%s, %s) → %s", (param, alias, expected) => {
        expect(mathNameWithRef(param, alias)).toBe(expected)
    })
})
