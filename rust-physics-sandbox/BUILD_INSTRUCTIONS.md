# Build Instructions

## Prerequisites
1.  **Rust**: Install from [rustup.rs](https://rustup.rs).
2.  **WebAssembly Target**: Run `rustup target add wasm32-unknown-unknown`.
3.  **Wasm-bindgen CLI**:
    *   Download the `0.2.90` release from [GitHub](https://github.com/rustwasm/wasm-bindgen/releases/tag/0.2.90).
    *   Extract it and ensure `wasm-bindgen` is in your PATH.
    *   *Alternatively*, run `cargo install wasm-bindgen-cli --version 0.2.90`.

## Building the Sandbox

1.  Navigate to the project directory:
    ```bash
    cd rust-physics-sandbox
    ```

2.  Compile the Rust code:
    ```bash
    cargo build --target wasm32-unknown-unknown --release
    ```

3.  Generate the JavaScript bindings:
    ```bash
    wasm-bindgen target/wasm32-unknown-unknown/release/rust_physics_sandbox.wasm \
      --out-dir static/pkg \
      --target web
    ```

## Running the Demo

1.  Start the local server:
    ```bash
    python3 serve.py
    ```

2.  Open your browser (Chrome/Edge/Firefox) and go to:
    `http://localhost:8080`

    You should see the simulation running. Click "Spawn Box" to add objects.

## Running with Elide (Orchestration Mode)

1.  Ensure you have **Elide** (GraalVM based runtime) installed.
2.  Navigate to the `elide` directory.
3.  Run the orchestrator:
    ```bash
    elide run orchestrator.py
    ```
    *Note: The current `orchestrator.py` is a mock. To fully enable WASM loading in Elide, ensure your Elide version supports the Polyglot WASM API and modify the script to load `../target/.../rust_physics_sandbox.wasm`.*
