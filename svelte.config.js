import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/**
 * Svelte configuration. `vitePreprocess` lets .svelte files contain
 * TypeScript, which is what svelte-check and the vite build both need.
 *
 * The style half of the preprocessor is switched off for the tests. It
 * has to resolve a Vite configuration of its own to run, and it cannot
 * do that inside the test runner. Every stylesheet in this project is
 * plain CSS, which the Svelte compiler compiles on its own, and the
 * tests do not look at the styles at all.
 */
const isTest =
  process.env.VITEST === "true" ||
  process.env.NODE_ENV === "test" ||
  process.argv.some((argument) => argument.includes("vitest"));

export default {
  preprocess: vitePreprocess({
    script: true,
    style: !isTest,
  }),
};
