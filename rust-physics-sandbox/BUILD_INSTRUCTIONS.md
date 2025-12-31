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
2.  Install `uv` (Fast Python Package Installer):
    ```bash
    curl -LsSf https://astral.sh/uv/install.sh | sh
    ```
3.  Navigate to the `elide` directory.
4.  Run the orchestrator using `uv`:
    ```bash
    uv run orchestrator.py
    ```
    *Note: The script automatically detects if it is running in a standard Python environment (and runs in mock mode) or in Elide (where it attempts to load the WASM module).*
