import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import pluginReact from "eslint-plugin-react";
import pluginImport from "eslint-plugin-import";

/** @type {import('eslint').Linter.Config[]} */
export default [
  {
    files: ["**/*.{js,mjs,cjs,ts,jsx,tsx}"],
  },
  { languageOptions: { globals: globals.browser } },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
  {
    ...pluginReact.configs.flat.recommended,
    settings: {
      react: {
        version: "detect",
      },
    },
    rules: {
      ...pluginReact.configs.flat.recommended.rules,
      "react/react-in-jsx-scope": "off",
      "react/jsx-uses-react": "off",
      // Disable base ESLint rule (TypeScript ESLint handles this)
      "no-unused-vars": "off",
    },
  },
  {
    // Override TypeScript ESLint unused variable rules to warnings
    // Note: If this rule doesn't exist in your version, the recommended config
    // may already handle it, or you may need to check available rules
    rules: {
      // Try the standard rule name first
      "@typescript-eslint/no-unused-vars": "warn",
      // If that doesn't work, the recommended config should already have it set
    },
  },
  {
    plugins: {
      import: pluginImport,
    },
    rules: {
      "import/no-unresolved": [
        "error",
        {
          commonjs: true,
          amd: false,
          caseSensitive: true,
        },
      ],
    },
    settings: {
      "import/resolver": {
        typescript: {
          alwaysTryTypes: true,
          project: "./tsconfig.json",
        },
        node: {
          extensions: [".js", ".jsx", ".ts", ".tsx"],
        },
      },
    },
  },
];
