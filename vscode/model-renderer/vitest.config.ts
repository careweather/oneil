import { defineConfig } from "vitest/config"

export default defineConfig({
    test: {
        // Pure utility tests — no DOM/browser APIs needed.
        environment: "node",
        include: ["src/**/*.test.ts"],
    },
})
