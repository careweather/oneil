/**
 * Minimal type declaration for the `citation-js` CommonJS package.
 * Only the parts used by `bibliography.ts` are typed here.
 */
declare module "citation-js" {
    interface CslEntry {
        id: string
        author?: { family?: string; given?: string; literal?: string }[]
        issued?: { "date-parts"?: [[number, ...number[]]] }
        title?: string
        [key: string]: unknown
    }

    class Cite {
        constructor(input: string)
        data: CslEntry[]
    }

    export = Cite
}

declare module "@citation-js/plugin-bibtex" {
    const plugin: unknown
    export default plugin
}
