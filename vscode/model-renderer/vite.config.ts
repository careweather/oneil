import { createRequire } from "module"
import { dirname, resolve } from "path"
import { defineConfig, type Plugin } from "vite"
import react from "@vitejs/plugin-react"

const require = createRequire(import.meta.url)

/**
 * Resolves `dep` as installed for `fromPackage`, including when npm nests it
 * under that package instead of hoisting it.
 */
function resolvedDepFrom(fromPackage: string, dep: string): string {
    const fromDir = dirname(require.resolve(`${fromPackage}/package.json`))
    return dirname(require.resolve(`${dep}/package.json`, { paths: [fromDir] }))
}

/**
 * Rollup plugin that removes legacy TTF and WOFF KaTeX font variants from the
 * bundle, keeping only WOFF2.  The VS Code webview runs on Chromium, which has
 * full WOFF2 support, so the legacy formats are dead weight.  The KaTeX CSS
 * still references them but the browser picks WOFF2 first and never requests
 * the missing files.
 */
function dropLegacyKaTeXFonts(): Plugin {
    return {
        name: "drop-legacy-katex-fonts",
        generateBundle(_options, bundle) {
            for (const key of Object.keys(bundle)) {
                if (/KaTeX[^/]*\.(ttf|woff)$/.test(key)) {
                    delete bundle[key]
                }
            }
        },
    }
}

// https://vite.dev/config/
export default defineConfig({
    plugins: [react(), dropLegacyKaTeXFonts()],
    resolve: {
        alias: {
            // react-pdf depends on a specific pdfjs-dist. Aliasing ensures that
            // `new URL("pdfjs-dist/...", import.meta.url)` in pdfWorker.ts
            // resolves to that same copy, preventing the "API version does not
            // match Worker version" error.
            "pdfjs-dist": resolvedDepFrom("react-pdf", "pdfjs-dist"),
        },
    },
    // Use relative asset paths so KaTeX fonts resolve correctly inside the
    // VS Code webview, which doesn't have a real web server root.
    base: "./",
    build: {
        // Output into the extension's out directory so the panel can load it.
        outDir: resolve(import.meta.dirname, "../out/model-renderer"),
        emptyOutDir: true,
        // This is a VS Code webview loaded from disk, not a web app served
        // over a network, so Vite's default 500 kB threshold is not meaningful.
        chunkSizeWarningLimit: 4000,
        // Never inline the PDF.js worker as a data URI — it must be emitted as
        // a separate file so VS Code's webview can serve it via localResourceRoots.
        assetsInlineLimit: 0,
        rollupOptions: {
            input: resolve(import.meta.dirname, "index.html"),
            output: {
                // Single deterministic filenames — the panel HTML references
                // these exact paths via vscode.Uri.joinPath.
                entryFileNames: "assets/index.js",
                chunkFileNames: "assets/[name].js",
                // All CSS is collected into one deterministic file so the
                // webview HTML can reference it at a known path.
                assetFileNames: (info) =>
                    info.name?.endsWith(".css") ? "assets/index.css" : "assets/[name].[ext]",
            },
        },
    },
})
