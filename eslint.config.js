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
    // Nuo P4.0.1 workspace split'o `target/` yra repo šaknyje (ne `src-tauri/target/`, kuris
    // nebeegzistuoja), o `gen/` — `crates/nullbyte-app/gen/`. Seni keliai reiškė, kad ESLint
    // realiai lindo į `target/**/out/__global-api-script.js` (Tauri build.rs sugeneruotą JS)
    // ir dėl to lūžo — pastebėta tik dabar, kai CI (P4.0.1 metu pasenęs) pirmą kartą realiai
    // pasiekė šį žingsnį po P4.0.3 CI pataisos.
    ignores: [
      "build/",
      ".svelte-kit/",
      "package/",
      "target/",
      "crates/nullbyte-app/gen/",
      "node_modules/",
    ],
  },
);
