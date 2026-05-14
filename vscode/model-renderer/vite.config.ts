import { defineConfig } from "vite"
import react from "@vitejs/plugin-react"
import { resolve } from "path"

// https://vite.dev/config/
export default defineConfig({
    plugins: [react()],
    resolve: {
        alias: {
            // react-pdf bundles its own pdfjs-dist internally.  Aliasing ensures
            // that `new URL("pdfjs-dist/...", import.meta.url)` in PdfPane.tsx
            // resolves to the same version react-pdf uses (currently 5.4.296),
            // preventing the "API version does not match Worker version" error.
            "pdfjs-dist": resolve(
                __dirname,
                "node_modules/react-pdf/node_modules/pdfjs-dist",
            ),
        },
    },
    // Use relative asset paths so KaTeX fonts resolve correctly inside the
    // VS Code webview, which doesn't have a real web server root.
    base: "./",
    build: {
        // Output into the extension's out directory so the panel can load it.
        outDir: resolve(__dirname, "../out/model-renderer"),
        emptyOutDir: true,
        // This is a VS Code webview loaded from disk, not a web app served
        // over a network, so Vite's default 500 kB threshold is not meaningful.
        chunkSizeWarningLimit: 4000,
        // Never inline the PDF.js worker as a data URI — it must be emitted as
        // a separate file so VS Code's webview can serve it via localResourceRoots.
        assetsInlineLimit: 0,
        rollupOptions: {
            input: resolve(__dirname, "index.html"),
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
