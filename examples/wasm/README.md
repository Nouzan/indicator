# Example: WASM Operator

`indicator` supports `WebAssembly`, and here is an example for building an
operator with `indicator` and
[`wasm_bindgen`](https://crates.io/crates/wasm-bindgen) and compiling it to
WebAssembly.

The source code of the operator can be found [here](../src/wasm.rs).

## Requirements

To run this example, make sure that you have installed
[`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/) and
[`deno`](https://deno.com/).

## Running

Run the following code under the `examples/wasm` dir:

```bash
deno task run
```

It will start compiling the operator to a wasm with `wasm-pack` and run a
`TypeScript` script that import and use it after the compilation.
