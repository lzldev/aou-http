/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface AouOptions {
  json?: boolean
}
export type Request = AouRequest
export declare class AouRequest {
  static fromString(request: string): Request
  get method(): string
  get path(): string
  get httpVersion(): string
  get headers(): Record<string, string>
  get body(): string
}
export declare class AouInstance { }
export declare class AouServer {
  constructor(options?: AouOptions | undefined | null)
  get(route:string,: handler:( request: AouRequest) => void): void
  listen(host: string, port: number): Promise<AouInstance>
}
