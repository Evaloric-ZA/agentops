import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2022",
    minify: process.env.TAURI_DEBUG ? false : "esbuild",
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/tests/**/*.test.ts"],
    // The test runner resolves every module with the "node" condition,
    // which picks Svelte's server build, where `mount` does not exist
    // and a store is not reactive. The bundler picks the browser
    // condition, which picks the client build that a component test
    // needs. These two entries are the only bare Svelte specifiers the
    // application uses.
    alias: [
      {
        find: /^svelte$/,
        replacement: fileURLToPath(
          new URL("./node_modules/svelte/src/index-client.js", import.meta.url),
        ),
      },
      {
        find: /^svelte\/store$/,
        replacement: fileURLToPath(
          new URL("./node_modules/svelte/src/store/index-client.js", import.meta.url),
        ),
      },
    ],
  },
});
