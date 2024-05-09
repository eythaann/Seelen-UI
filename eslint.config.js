const stylistic = require('@stylistic/eslint-plugin');
const simpleImportSort = require('eslint-plugin-simple-import-sort');
const tsEslint = require('typescript-eslint');

module.exports = [
  {
    ignores: ['node_modules/', '.git/', 'dist/', 'target/'],
  },
  {
    files: ['**/*.{js,jsx,ts,tsx,mjs}'],
    plugins: {
      '@stylistic': stylistic,
      'simple-import-sort': simpleImportSort,
      '@ts': tsEslint.plugin,
    },
    languageOptions: {
      parser: tsEslint.parser,
    },
    rules: {
      'simple-import-sort/imports': [
        'error',
        {
          groups: [
            [''],
            ['.*/(infra|infrastructure).*'],
            ['.*/app'],
            ['.*/domain.*'],
            ['.*.module.css$'],
            ['.*.css$'],
          ],
        },
      ],
      'no-dupe-keys': 'error',
      '@stylistic/key-spacing': ['error', { beforeColon: false }],
      '@stylistic/block-spacing': 'error',
      '@stylistic/arrow-spacing': 'error',
      '@stylistic/one-var-declaration-per-line': ['error', 'always'],
      '@stylistic/object-curly-spacing': ['error', 'always'],

      '@stylistic/brace-style': ['error', '1tbs'],
      '@stylistic/jsx-quotes': ['error', 'prefer-double'],
      'no-nested-ternary': 'error',

      '@stylistic/comma-dangle': ['error', 'always-multiline'],
      '@stylistic/comma-spacing': [
        'error',
        {
          before: false,
          after: true,
        },
      ],
      '@stylistic/keyword-spacing': 'error',
      '@stylistic/space-before-blocks': 'error',
      '@stylistic/no-multiple-empty-lines': [
        'error',
        {
          max: 1,
          maxEOF: 1,
        },
      ],
      '@stylistic/lines-between-class-members': [
        'error',
        'always',
        { exceptAfterSingleLine: true },
      ],
      '@stylistic/padded-blocks': ['error', 'never'],
      '@stylistic/arrow-parens': ['error', 'always'],
      '@stylistic/space-before-function-paren': [
        'error',
        {
          anonymous: 'always',
          named: 'never',
          asyncArrow: 'always',
        },
      ],
      '@stylistic/quotes': ['error', 'single'],
      '@stylistic/semi': 'error',
      '@stylistic/no-multi-spaces': ['error'],
      '@stylistic/no-trailing-spaces': ['error'],
      '@stylistic/space-infix-ops': ['error'],
      '@stylistic/indent': ['error', 2],
      '@stylistic/jsx-indent': ['error', 2],
      '@stylistic/member-delimiter-style': ['error'],
      '@stylistic/type-annotation-spacing': ['error'],
      '@ts/no-unused-vars': [
        'warn',
        {
          varsIgnorePattern: '^_',
          argsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          destructuredArrayIgnorePattern: '^_',
        },
      ],
    },
  },
];
