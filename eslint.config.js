import js from '@eslint/js';
import {defineConfig} from 'eslint/config';
import googleConfig from 'eslint-config-google';
import globals from 'globals';

export default defineConfig([
  {
    files: ['cubicle/**/*.js'],

    plugins: {
      js,
    },
    extends: [
      'js/recommended',
      googleConfig,
    ],

    languageOptions: {
      ecmaVersion: 11,
      sourceType: 'module',
      globals: {
        ...globals.browser,
        ...globals.webextensions,
      },
    },
  },
  {
    ignores: ['cubicle/res/components/context-map.js'],
  },
]);
