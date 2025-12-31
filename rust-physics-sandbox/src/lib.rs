mod physics;
mod render;
mod utils;
mod soft_body;

use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct SimCommand {
    cmd: String,
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
}

thread_local! {
    static CMD_QUEUE: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct Simulation {
    physics: physics::PhysicsWorld,
    // Renderer is optional because in headless (Elide) mode we might not have it
    renderer: Option<render::Renderer>,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Simulation {
        utils::set_panic_hook();
        Simulation {
            physics: physics::PhysicsWorld::new(),
            renderer: None,
        }
    }

    pub async fn init_graphics(&mut self, canvas_id: &str) -> Result<(), JsValue> {
        let renderer = render::Renderer::new(canvas_id).await?;
        self.renderer = Some(renderer);
        Ok(())
    }

    // Connect to WebSocket orchestrator
    pub fn connect_orchestrator(&self) -> Result<(), JsValue> {
        let ws = WebSocket::new("ws://localhost:8765")?;
        
        // We need a way to call methods on 'self' from the closure.
        // However, 'self' is passed by value or reference to WASM, handling lifetimes is tricky.
        // The standard pattern is to use a global or a handle.
        // Since 'Simulation' is owned by JS, we can't easily capture 'self' in a closure that lives longer.
        // 
        // Alternative: The WebSocket puts messages into a queue, and 'step()' processes them.
        // This avoids callback hell and borrowing issues.
        
        // Let's attach the queue to the Simulation struct? 
        // But Simulation is moved to JS.
        // We can use a static RefCell queue? Or just pass a closure that calls a global function?
        
        // Ideally, we want: on_message -> queue.push(msg)
        // step() -> while let Some(msg) = queue.pop() { handle(msg) }
        
        // Hack: Use a window-level queue in JS, or just use a static mutex in Rust.
        // Let's use a static queue for simplicity in this demo.
        
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let txt: String = txt.into();
                // Push to global queue
                CMD_QUEUE.with(|q| q.borrow_mut().push(txt));
            }
        });
        
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget(); // Leak memory to keep handler alive
        
        Ok(())
    }

    pub fn step(&mut self) {
        // Process Commands
        CMD_QUEUE.with(|q| {
            let mut queue = q.borrow_mut();
            for cmd_str in queue.drain(..) {
                if let Ok(cmd) = serde_json::from_str::<SimCommand>(&cmd_str) {
                    match cmd.cmd.as_str() {
                        "spawn_box" => self.physics.spawn_box(cmd.x.unwrap_or(0.0), cmd.y.unwrap_or(5.0), cmd.z.unwrap_or(0.0)),
                        "spawn_sphere" => self.physics.spawn_sphere(cmd.x.unwrap_or(0.0), cmd.y.unwrap_or(5.0), cmd.z.unwrap_or(0.0)),
                        "spawn_liquid" => self.physics.spawn_liquid(cmd.x.unwrap_or(0.0), cmd.y.unwrap_or(5.0), cmd.z.unwrap_or(0.0)),
                        "spawn_cloth" => self.physics.spawn_cloth(cmd.x.unwrap_or(0.0), cmd.y.unwrap_or(5.0), cmd.z.unwrap_or(0.0), 10, 10),
                        "spawn_avalanche" => self.physics.spawn_avalanche(cmd.x.unwrap_or(0.0), cmd.y.unwrap_or(5.0), cmd.z.unwrap_or(0.0)),
                        "spawn_tsunami" => self.physics.spawn_tsunami(cmd.x.unwrap_or(0.0), cmd.y.unwrap_or(5.0), cmd.z.unwrap_or(0.0)),
                        _ => log(&format!("Unknown command: {}", cmd.cmd)),
                    }
                }
            }
        });

        self.physics.step();
        
        if let Some(renderer) = &mut self.renderer {
            // Sync physics state to renderer
            // In a real impl, we'd pass positions here
            renderer.render(&self.physics);
        }
    }
    
    pub fn spawn_box(&mut self, x: f32, y: f32, z: f32) {
        self.physics.spawn_box(x, y, z);
    }

    pub fn spawn_sphere(&mut self, x: f32, y: f32, z: f32) {
        self.physics.spawn_sphere(x, y, z);
    }
    
    pub fn spawn_floor(&mut self) {
        self.physics.spawn_floor();
    }
    
    pub fn spawn_liquid(&mut self, x: f32, y: f32, z: f32) {
        self.physics.spawn_liquid(x, y, z);
    }
    
    pub fn spawn_cloth(&mut self, x: f32, y: f32, z: f32) {
        self.physics.spawn_cloth(x, y, z, 10, 10);
    }

    pub fn spawn_avalanche(&mut self, x: f32, y: f32, z: f32) {
        self.physics.spawn_avalanche(x, y, z);
    }
    
    pub fn spawn_tsunami(&mut self, x: f32, y: f32, z: f32) {
        self.physics.spawn_tsunami(x, y, z);
    }

    pub fn get_first_object_y(&self) -> f32 {
        self.physics.get_first_object_y()
    }
    
    pub fn update_camera(&mut self, dx: f32, dy: f32, zoom: f32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.update_camera(dx, dy, zoom);
        }
    }
    
    pub fn handle_click(&mut self, x: f32, y: f32) {
        if let Some(renderer) = &self.renderer {
            let (origin, dir) = renderer.get_ray_from_screen(x, y);
            if let Some(handle) = self.physics.cast_ray(origin.x, origin.y, origin.z, dir.x, dir.y, dir.z) {
                // For now, just apply a vertical impulse to the clicked object to show selection
                self.physics.apply_impulse(handle, 0.0, 5.0, 0.0);
            }
        }
    }
}
