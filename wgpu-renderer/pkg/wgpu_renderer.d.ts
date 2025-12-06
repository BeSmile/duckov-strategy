/* tslint:disable */
/* eslint-disable */
export function run_web(): void;
export function get_maps(): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly run_web: () => [number, number];
  readonly get_maps: () => any;
  readonly wasm_bindgen__convert__closures_____invoke__h14f2b1bcb49addf5: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__h3f896890c00791e0: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hfc65c76fa50e02e4: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h0b7e2fe61ee109a1: (a: number, b: number, c: any, d: any) => void;
  readonly wasm_bindgen__convert__closures_____invoke__h246b7edee645cc6d: (a: number, b: number) => void;
  readonly wasm_bindgen__closure__destroy__h3fc80d037a8e410f: (a: number, b: number) => void;
  readonly wasm_bindgen__convert__closures_____invoke__hd9c4eb179323d204: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__ha18cfd18a95c845f: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
