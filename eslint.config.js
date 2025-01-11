const { aiou } = require('@aiou/eslint-config')

module.exports = aiou({ ssr: false }, [
  {
    ignores: [
      '**/fixtures/**',
      '**/*.d.ts',
      '**/binding_node/*.js',
      '**/__dir_snapshots__/**',
    ],
  },
])
