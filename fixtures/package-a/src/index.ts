import './global.css'
import './styles/room.scss'
import './query.css?query'
import styles from './css.module.css'
import vite from './vite.svg'
import { name } from '@demo/package-b'
import { table } from 'functional-md'
import * as reexport from './re-export'
import { Button, IconButton } from '@mui/material'

console.log(Button, IconButton)
console.log(styles, vite)
console.log(name)
console.log(table)

const a = 1

export { reexport }
export default a
export { another } from './another'
