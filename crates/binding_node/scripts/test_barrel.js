const { preOptimize } = require('../index')
const path = require('node:path')

const main = () => {
  const root = path.join(__dirname, '../../../fixtures/package-a')
  preOptimize({ root, barrelPackages: ['@mui/material'] })
}

main()
