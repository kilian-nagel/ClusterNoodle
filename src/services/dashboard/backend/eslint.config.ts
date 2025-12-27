import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import { defineConfig } from "eslint/config";
import pluginSecurity from "eslint-plugin-security";
import noSecretsPlugin from "eslint-plugin-no-secrets";
import pluginSecurityNode from 'eslint-plugin-security';

export default defineConfig([
  {
    languageOptions: {
      globals: {
        ...globals.node,
      }
    }
  },
  {
    files: ["**/*.{js,mjs,cjs,ts,mts,cts}"],
    plugins: {
      js,
      security: pluginSecurity,
      "no-secrets": noSecretsPlugin,
      "security-node": pluginSecurityNode,
    },
    rules: {
      // Base recommended rules
      ...js.configs.recommended.rules,
      
      // Security plugin rules - detects common security issues
      "security/detect-buffer-noassert": "error",
      "security/detect-child-process": "warn",
      "security/detect-disable-mustache-escape": "error",
      "security/detect-eval-with-expression": "error",
      "security/detect-new-buffer": "error",
      "security/detect-no-csrf-before-method-override": "error",
      "security/detect-non-literal-fs-filename": "warn",
      "security/detect-non-literal-regexp": "warn",
      "security/detect-non-literal-require": "warn",
      "security/detect-object-injection": "warn",
      "security/detect-possible-timing-attacks": "warn",
      "security/detect-pseudoRandomBytes": "error",
      "security/detect-unsafe-regex": "error",
      
      // No secrets - prevents committing secrets
      "no-secrets/no-secrets": ["error", { "tolerance": 4.5 }],
      
      // Security Node - Node.js specific security
      "security-node/detect-eval-with-expression": "error",
      
      // Additional TypeScript security rules
      "@typescript-eslint/no-explicit-any": "warn",
      
      // Prevent dangerous patterns
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-new-func": "error",
      "no-script-url": "error",
    },
    languageOptions: {
      globals: globals.node
    }
  },
  ...tseslint.configs.recommended,
]);