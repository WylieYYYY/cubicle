import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import globals from 'globals';

export default defineConfig([
    {
        files: ['**/*.js'],

        plugins: {
            js,
        },
        extends: [
            'js/recommended',
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
])