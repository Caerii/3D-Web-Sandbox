import time
import asyncio
import json
import websockets

# NOTE: This script is designed to run in the Elide runtime (GraalVM) or Python.
# It acts as a WebSocket server to orchestrate the browser simulation.

async def handler(websocket):
    print("Browser connected! Starting simulation scenario...")
    
    # 1. Reset / Setup
    # We could send a "RESET" command if we implemented it.
    
    # 2. Build a Tower
    print("Agent: Building tower...")
    for i in range(5):
        await websocket.send(json.dumps({
            "cmd": "spawn_box",
            "x": 0.0,
            "y": 0.5 + i * 1.1,
            "z": 0.0
        }))
        await asyncio.sleep(0.5) 
        
    # 3. Spawn Cloth
    print("Agent: Spawning Cloth...")
    await websocket.send(json.dumps({
        "cmd": "spawn_cloth",
        "x": 0.0,
        "y": 8.0,
        "z": 0.0
    }))
    await asyncio.sleep(2.0)
    
    # 4. Pour Water
    print("Agent: Pouring water to knock down tower!")
    await websocket.send(json.dumps({
        "cmd": "spawn_liquid",
        "x": 0.5,
        "y": 8.0,
        "z": 0.0
    }))
    
    # Keep connection alive to receive logs?
    # In this simple version, we just wait.
    await asyncio.sleep(10.0)
    print("Scenario complete.")

async def main():
    print("Starting Orchestrator WebSocket Server on ws://localhost:8765")
    async with websockets.serve(handler, "localhost", 8765):
        await asyncio.Future()  # run forever

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        pass
