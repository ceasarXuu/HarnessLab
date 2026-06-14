import js from '@eslint/js'
import skipFormatting from '@vue/eslint-config-prettier/skip-formatting'
import { defineConfigWithVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'
import globals from 'globals'
import pluginVue from 'eslint-plugin-vue'

export default defineConfigWithVueTs(
  {
    ignores: ['coverage', 'dist', 'playwright-report', 'storybook-static', 'test-results'],
  },
  js.configs.recommended,
  pluginVue.configs['flat/recommended'],
  vueTsConfigs.recommended,
  {
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.node,
      },
    },
    rules: {
      'vue/multi-word-component-names': 'off',
    },
  },
  skipFormatting,
)

