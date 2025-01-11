import path from 'node:path'

import {
  describe,
  expect,
  it,
} from 'vitest'
import { toMatchDir } from 'vitest-extra'

import { transform } from '../index.js'

expect.extend({ toMatchDir })

describe('transform', () => {
  it('base', async () => {
    const root = path.join(__dirname, '../../../fixtures/package-a')
    transform({ root, barrelPackages: [] })
    // TODO: replace root with placeholder
    expect(path.join(root, 'dist')).toMatchDir()
  })
})
