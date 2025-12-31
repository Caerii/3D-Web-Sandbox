# Empirical Performance Characteristics

This document outlines the performance profile of the Rust + WebAssembly + WebGPU physics sandbox.

## Architectural Advantages

1.  **WebAssembly (WASM)**:
    *   **Near-Native Execution**: The physics simulation (Rapier3D, Salva3D) compiles to WASM, running at near-native speeds compared to JavaScript-based physics engines.
    *   **No Garbage Collection**: Rust's memory management avoids the random stuttering (GC pauses) common in complex JS simulations.

2.  **WebGPU Rendering**:
    *   **Instanced Rendering**: We use hardware instancing to draw thousands of objects (cubes, particles) with a single draw call. This drastically reduces CPU-overhead.
    *   **Compute Potential**: Future optimizations can move the physics simulation itself to the GPU using Compute Shaders, though currently, it runs on the CPU (WASM).

3.  **Zero-Copy Networking (Internal)**:
    *   Bytemuck and flat memory buffers allow us to move simulation data from Rust directly to the GPU buffers with minimal serialization overhead.

## Benchmark Scenarios

Use the implemented "Avalanche" and "Tsunami" buttons to stress test the system.

### 1. The "Avalanche" (Rigid Body Stress)
*   **Scenario**: 500+ dynamic rigid bodies interacting with complex collisions.
*   **Bottleneck**: Broad-phase collision detection (CPU/WASM).
*   **Expected Performance (Modern Laptop)**:
    *   **500 Objects**: 60 FPS (Silky smooth)
    *   **2,000 Objects**: 30-50 FPS (Playable)
    *   **5,000+ Objects**: < 20 FPS (Simulation steps take longer than 16ms)

### 2. The "Tsunami" (Fluid Particle Stress)
*   **Scenario**: 2,000+ SPH (Smoothed Particle Hydrodynamics) fluid particles.
*   **Bottleneck**: Neighbor searching for fluid density calculations (CPU/WASM).
*   **Expected Performance**:
    *   **1,000 Particles**: 60 FPS
    *   **3,000 Particles**: 30 FPS
    *   **10,000 Particles**: < 10 FPS (SPH is computationally expensive: O(N*neighbors))

### 3. The "Curtain" (Soft Body Stress)
*   **Scenario**: Mass-spring cloth simulation.
*   **Bottleneck**: Constraint solving iterations.
*   **Expected Performance**:
    *   **10x10 Grid (100 Nodes)**: 60 FPS
    *   **50x50 Grid (2,500 Nodes)**: 30-45 FPS

## Your Empirical Results

Record your findings here after running the simulation on your hardware.

| Hardware | Scenario | Object Count | FPS | Notes |
| :--- | :--- | :--- | :--- | :--- |
| *e.g., MacBook Pro M1* | Idle | 0 | 60 | Baseline |
| | Avalanche | 500 Boxes | | |
| | Tsunami | 2000 Particles | | |
| | Combined | 500 Box + 2k Water | | |

## Optimization Path

If performance drops below targets, the following optimizations are available:

1.  **Multithreading**: Enable `SharedArrayBuffer` to allow Rayon in WASM (requires specific HTTP headers).
2.  **GPU Physics**: Move particle physics (Salva) entirely to WebGPU Compute Shaders.
3.  **Substepping**: Reduce physics accuracy for visual smoothness (lower sub-steps).
