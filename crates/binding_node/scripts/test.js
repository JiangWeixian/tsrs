const { transform } = require('../index')
const path = require('node:path')

const main = () => {
  const root = path.join(__dirname, '../../../fixtures/package-a')
  transform({
    root,
    optimize: {
      barrelPackages: ['@mui/material'],
    },
  })
  // transform({ root, barrelPackages: [] })
}

main()
