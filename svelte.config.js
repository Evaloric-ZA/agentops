import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** Svelte configuration. `vitePreprocess` lets .svelte files contain
 *  TypeScript, which is what svelte-check and the vite build both need. */
export default {
  preprocess: vitePreprocess(),
};
