mod physics;
mod render;
mod utils;

use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

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

    pub fn step(&mut self) {
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

    pub fn get_first_object_y(&self) -> f32 {
        self.physics.get_first_object_y()
    }
}
