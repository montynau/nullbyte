import js from "@eslint/js";
import prettier from "eslint-config-prettier";
import svelte from "eslint-plugin-svelte";
import globals from "globals";
import ts from "typescript-eslint";

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs.recommended,
  prettier,
  ...svelte.configs.prettier,
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
  },
  {
    files: ["**/*.svelte"],
    languageOptions: {
      parserOptions: {
        parser: ts.parser,
        extraFileExtensions: [".svelte"],
      },
    },
  },
  {
    // shadcn-svelte generuoti komponentai — generinis `href` prop, ne app-lygio
    // SvelteKit maršrutas, todėl resolve() reikalavimas čia netaikytinas (CLAUDE.md §7.4).
    files: ["src/lib/components/ui/**"],
    rules: {
      "svelte/no-navigation-without-resolve": "off",
    },
  },
  {
    ignores: [
      "build/",
      ".svelte-kit/",
      "package/",
      "src-tauri/target/",
      "src-tauri/gen/",
      "node_modules/",
    ],
  },
);
