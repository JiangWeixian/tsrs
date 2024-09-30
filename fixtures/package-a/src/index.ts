import './global.css'
import './query.css?query'
import { name } from '@demo/package-b'
import { table } from 'functional-md'
import * as reexport from './re-export'

console.log(name)
console.log(table)

const a = 1

export { reexport }
export default a
export { another } from './another'
