# Technical Design Document: Rust-Based 3D Physics Sandbox with Elide Integration

## 1. Overview
This project involves building a high-performance 3D physics sandbox in Rust, targeting WebAssembly (WASM) for browser deployment and leveraging WebGPU for hardware-accelerated rendering. A key differentiator is the integration with Elide, a polyglot runtime, to demonstrate multi-language agent coordination (e.g., Python scripts controlling the Rust simulation) without traditional network IPC overhead.

## 2. System Architecture

The system is composed of three main subsystems:

### 2.1. Physics Simulation Core (Rust/WASM)
*   **Role:** Performs all physics calculations including rigid body dynamics, soft body deformations, and fluid dynamics.
*   **Technology:** Rust compiled to `wasm32-unknown-unknown`.
*   **Libraries:**
    *   **Rigid Bodies:** `rapier3d` (Dimforge). deterministic, fast, WASM-ready.
    *   **Fluid Dynamics:** `salva3d` (Dimforge) for SPH particle fluids.
    *   **Soft Bodies:** Custom PBD (Position-Based Dynamics) implementation or simplified spring-mass systems coupled with Rapier.
*   **Design:**
    *   Exposes a high-level API via `wasm-bindgen` for initialization, stepping, and state manipulation.
    *   Decoupled from rendering: The physics engine outputs state (transforms, particle positions), which the renderer consumes.

### 2.2. Rendering Engine (Rust via WebGPU)
*   **Role:** Visualizes the simulation in real-time.
*   **Technology:** `wgpu` (Rust implementation of WebGPU).
*   **Design:**
    *   **WebGPU First:** Uses compute shaders (where applicable) and modern pipeline for maximum performance.
    *   **WebGL Fallback:** `wgpu` supports a WebGL backend for broad compatibility.
    *   **Data Flow:** Reads physics state each frame, updates GPU buffers (instances, vertex data), and issues draw calls.
    *   **Loop:** Can run in the browser's `requestAnimationFrame` loop.

### 2.3. Elide Polyglot Orchestration
*   **Role:** Coordinators/Agents that control the simulation logic.
*   **Technology:** Elide (GraalVM-based runtime).
*   **Integration:**
    *   The Rust WASM module is loaded into Elide's runtime (via GraalWasm).
    *   **Direct Interop:** Python/JS scripts running in Elide call exported Rust functions directly. No JSON serialization or local sockets required.
    *   **Use Case:** A Python script ("Agent") can query simulation state and apply forces programmatically (e.g., "keep this box upright").

## 3. Technology Stack & Library Choices

| Component | Technology | Justification |
| :--- | :--- | :--- |
| **Language** | Rust (Edition 2021+) | Memory safety, performance, excellent WASM tooling. |
| **Physics** | `rapier3d` | Best-in-class Rust physics, official WASM support, determinstic. |
| **Fluids** | `salva3d` | Integration with Rapier, SPH implementation. |
| **Graphics** | `wgpu` | Portable WebGPU implementation, future-proof, safe. |
| **Windowing** | `winit` | Cross-platform window handling (supports DOM on web). |
| **Build Tool** | `wasm-pack` | Standard for building Rust to NPM/Web compatible WASM. |
| **Runtime** | Elide (GraalVM) | Enables zero-overhead polyglot execution (Python/JS + Rust WASM). |

## 4. Runtime Design

### 4.1. Browser Mode (Standalone)
*   **Entry:** `index.html` loads the WASM module.
*   **Loop:** Browser `requestAnimationFrame` drives the Rust `step()` and `render()` functions.
*   **Input:** DOM events (mouse/keyboard) are passed to Rust (via `winit` or direct listeners).

### 4.2. Elide Mode (Orchestrated)
*   **Entry:** `elide run orchestrator.py`.
*   **Execution:** Elide loads the Rust WASM module.
*   **Logic:** The Python script drives the loop, calling `sim.step()` and `sim.apply_force(...)`.
*   **Visualization:** Headless simulation for logic verification, or streaming state to a separate view (optional extension).

---

# Implementation Roadmap

## Phase 1: Project Setup & Core Structure
1.  **Initialize Workspace:** Create a Rust library project.
    *   `cargo new rust-physics-sandbox --lib`
    *   Configure `Cargo.toml` with `crate-type = ["cdylib"]`.
2.  **Dependencies:** Add `rapier3d`, `wgpu`, `wasm-bindgen`, `winit`, `console_error_panic_hook`.
3.  **Build Pipeline:** Verify `wasm-pack build --target web` works.

## Phase 2: Physics Engine Implementation
1.  **World Setup:** Initialize `rapier3d::prelude::PhysicsPipeline`, `RigidBodySet`, `ColliderSet`.
2.  **Basic API:** Export `init()`, `step()`, `spawn_box()`, `get_object_positions()`.
3.  **Soft Body/Fluids:**
    *   Integrate `salva3d` for fluids.
    *   Implement basic soft-body structs (spring-mass) and integrate into the step loop.

## Phase 3: Rendering with WebGPU
1.  **WGPU Context:** Initialize `Instance`, `Adapter`, `Device`, `Queue`, `Surface`.
2.  **Render Loop:** Implement the main loop handling `Event::RedrawRequested`.
3.  **Pipelines:** Create render pipelines for:
    *   Rigid bodies (Mesh rendering).
    *   Fluids (Instanced sphere rendering or point sprites).
4.  **Sync:** Map physics positions to WGPU instance buffers.

## Phase 4: Interactivity & Polish
1.  **Camera:** Implement an orbital camera controller responding to mouse drag.
2.  **Picking:** Use Ray-casting from Rapier to select and drag objects.
3.  **UI:** Simple HTML overlay or `egui` for parameter tuning (gravity, time scale).

## Phase 5: Elide Integration
1.  **Scripting:** Write a Python script `orchestrator.py` that imports the WASM module.
2.  **Bindings:** Ensure Rust exports are friendly for GraalWasm (simple types).
3.  **Demo:** Create a scenario where a Python agent controls an object based on simulated sensor data.

## Phase 6: Documentation & Deployment
1.  **Docs:** Write instructions for building and running both modes.
2.  **Deploy:** Host static files (HTML/WASM) on a web server.
