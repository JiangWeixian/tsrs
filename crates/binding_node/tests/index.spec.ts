import path from 'node:path'

import { describe, it } from 'vitest'

import { transform } from '../index.js'

describe('optimize', (t) => {
  it('barrel', async () => {
    const root = path.join(__dirname, '../../../fixtures/package-a')
    await transform({ root, barrelPackages: ['@mui/material'] })
  })
})
