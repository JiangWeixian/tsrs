import path from 'node:path'

import {
  describe,
  expect,
  it,
} from 'vitest'
import { toMatchDir } from 'vitest-extra'

import { transform } from '../index.js'

expect.extend({ toMatchDir })

describe('optimize', () => {
  it('barrel', async () => {
    const root = path.join(__dirname, '../../../fixtures/package-a')
    transform({ root, barrelPackages: ['@mui/material'] })
    // TODO: replace root with placeholder
    // expect(path.join(root, 'dist')).toMatchDir()
  })
})
