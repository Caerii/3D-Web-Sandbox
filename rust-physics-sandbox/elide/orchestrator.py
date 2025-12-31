import time

# NOTE: This script is designed to run in the Elide runtime (GraalVM).
# Elide provides the 'polyglot' module implicitly or via import if configured.
# If running locally with Python (CPython), this will fail unless we mock it.

try:
    import polyglot
except ImportError:
    print("Warning: 'polyglot' module not found. Running in mock mode.")
    class MockPolyglot:
        def eval(self, file, mime_type):
            print(f"Loading {file} as {mime_type}...")
            return MockSource()
            
    class MockSource:
        def instantiate(self):
            return MockInstance()
            
    class MockInstance:
        def Simulation(self):
            return MockSim()
            
    class MockSim:
        def spawn_box(self, x, y, z):
            print(f"Rust: spawn_box({x}, {y}, {z})")
        def spawn_sphere(self, x, y, z):
            print(f"Rust: spawn_sphere({x}, {y}, {z})")
        def spawn_liquid(self, x, y, z):
            print(f"Rust: spawn_liquid({x}, {y}, {z})")
        def spawn_cloth(self, x, y, z, w, h):
            print(f"Rust: spawn_cloth({x}, {y}, {z})")
        def spawn_floor(self):
            print("Rust: spawn_floor()")
        def step(self):
            pass
        def get_first_object_y(self):
            return 5.0

    polyglot = MockPolyglot()

def main():
    print("Initializing Elide Orchestrator...")
    
    # Load the Rust WASM module
    # In a real Elide env, path might need adjustment relative to working dir
    wasm_path = "../target/wasm32-unknown-unknown/release/rust_physics_sandbox.wasm"
    
    try:
        source = polyglot.eval(file=wasm_path, mime_type="application/wasm")
    except Exception as e:
        # Fallback path if running from inside elide dir
        wasm_path = "target/wasm32-unknown-unknown/release/rust_physics_sandbox.wasm"
        # In mock mode, the file read isn't real, but real polyglot reads bytes or file path
        # Actually polyglot.eval(path=...) or polyglot.eval(string=...)
        # For WASM, GraalWasm supports loading binary.
        print(f"Assuming WASM loaded from {wasm_path}")
        source = polyglot.eval(file=wasm_path, mime_type="application/wasm")

    # Instantiate the module
    # Note: WASM modules in Graal usually export a 'main' or the exports object.
    # If using wasm-bindgen, the exports are a bit different (memory, functions).
    # However, Elide/GraalWasm can bridge this. 
    # For this demo, we assume the high-level class 'Simulation' is accessible 
    # (which requires binding support in the runtime or manual export mapping).
    # Since wasm-bindgen produces a JS wrapper, pure WASM usage in Polyglot 
    # typically accesses raw exports like 'new_simulation' (mangled name) 
    # or requires the JS wrapper to be loaded in a JS context that calls Python.
    
    # Simpler approach for pure Polyglot: Treat WASM as library.
    instance = source.instantiate()
    
    # In raw WASM (without JS glue), we'd call exports directly.
    # Our Rust code wraps things in a class 'Simulation'. 
    # The raw exports will have functions like 'simulation_new', 'simulation_step'.
    # For the sake of the demo script, we assume a clean binding.
    
    sim = instance.Simulation() 
    sim.spawn_floor()
    
    # Scenario: Build a tower
    print("Agent: Building tower...")
    for i in range(5):
        sim.spawn_box(0, 0.5 + i * 1.1, 0)
        time.sleep(0.1) # Brief pause to let physics settle slightly
        
    print("Agent: Spawning Cloth...")
    try:
        sim.spawn_cloth(0, 8, 0, 10, 10)
    except Exception:
        print("MockSim does not support spawn_cloth")

    print("Simulation started.")
    
    # Agent Loop
    count = 0
    while count < 300: 
        sim.step()
        
        # Read sensor data from Rust
        y_pos = sim.get_first_object_y()
        
        # Event: At tick 100, pour water
        if count == 100:
            print("Agent: Pouring water to knock down tower!")
            sim.spawn_liquid(0.5, 8, 0)
        
        # Log occasionally
        if count % 60 == 0:
            print(f"Agent: Tick {count}, Object Y={y_pos:.2f}")
        
        time.sleep(0.016) # ~60 FPS
        count += 1

if __name__ == "__main__":
    main()
