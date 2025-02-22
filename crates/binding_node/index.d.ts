/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface TransformOptimizeOptions {
  /** Optimized packages */
  barrelPackages?: Array<string>
}
export interface TransformOptions {
  root: string
  output?: string
  externals?: Array<string>
  exclude?: Array<string>
  modules?: Array<string>
  /** Optimized options */
  optimize: TransformOptimizeOptions
}
export declare function transform(options: TransformOptions): void
