use rapier3d::prelude::*;
use std::collections::HashMap;
use salva3d::integrations::rapier::FluidsPipeline;
use salva3d::LiquidWorld;
use salva3d::object::{Fluid, FluidHandle};
use nalgebra::Point3;

pub struct PhysicsWorld {
    pub pipeline: PhysicsPipeline,
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    
    // Salva Fluid Physics
    pub fluid_pipeline: FluidsPipeline,
    pub fluid_handle: FluidHandle, // Keep track of our main water body
    
    // Keep track of what we spawned to categorize them for rendering
    // Map RigidBodyHandle -> ObjectType (0: box, 1: sphere)
    pub object_types: HashMap<RigidBodyHandle, u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct RenderInfo {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub qx: f32,
    pub qy: f32,
    pub qz: f32,
    pub qw: f32,
    pub obj_type: u32,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        let particle_radius = 0.1;
        let smoothing_factor = 2.0;
        
        let mut fluid_pipeline = FluidsPipeline::new(particle_radius, smoothing_factor);
        
        // Create a default fluid (water)
        let fluid = Fluid::new(Vec::new(), particle_radius, 1000.0); // Density ~1000 kg/m^3
        let fluid_handle = fluid_pipeline.liquid_world.add_fluid(fluid);

        Self {
            pipeline: PhysicsPipeline::new(),
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            
            fluid_pipeline,
            fluid_handle,
            
            object_types: HashMap::new(),
        }
    }

    pub fn step(&mut self) {
        let physics_hooks = ();
        let event_handler = ();
        
        // Step Rigid Body Physics
        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &physics_hooks,
            &event_handler,
        );
        
        // Step Fluid Physics
        // Note: FluidsPipeline::step takes dt. Rapier uses integration_parameters.dt
        let dt = self.integration_parameters.dt;
        self.fluid_pipeline.step(
            &self.gravity,
            dt,
            &self.collider_set,
            &mut self.rigid_body_set,
        );
    }

    pub fn spawn_box(&mut self, x: f32, y: f32, z: f32) {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .build();
        let collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5).restitution(0.7).build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
        self.object_types.insert(body_handle, 0); // 0 = Box
    }
    
    pub fn spawn_floor(&mut self) {
        let rigid_body = RigidBodyBuilder::fixed().build();
        let collider = ColliderBuilder::cuboid(100.0, 0.1, 100.0).build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
        self.object_types.insert(body_handle, 0); // Floor is also a box shape
    }

    pub fn spawn_sphere(&mut self, x: f32, y: f32, z: f32) {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![x, y, z])
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let body_handle = self.rigid_body_set.insert(rigid_body);
        self.collider_set.insert_with_parent(collider, body_handle, &mut self.rigid_body_set);
        self.object_types.insert(body_handle, 1); // 1 = Sphere
    }

    pub fn spawn_liquid(&mut self, x: f32, y: f32, z: f32) {
        // Spawn a block of water
        // Resolution 0.2 means distance between particles (twice the radius)
        let n = 5; 
        let d = 0.2; 
        let mut particles = Vec::new();
        
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    particles.push(Point3::new(
                        x + (i as f32) * d, 
                        y + (j as f32) * d, 
                        z + (k as f32) * d
                    ));
                }
            }
        }
        
        // Add particles to our existing fluid
        let fluid = self.fluid_pipeline.liquid_world.fluids_mut().get_mut(self.fluid_handle).unwrap();
        fluid.add_particles(&particles, None);
    }

    // Helper for Elide agent to inspect state
    // Just returns the Y position of the first dynamic object found (for simple testing)
    pub fn get_first_object_y(&self) -> f32 {
        for (_handle, body) in self.rigid_body_set.iter() {
            if body.is_dynamic() {
                return body.translation().y;
            }
        }
        0.0
    }

    // Returns a flattened list of transforms: [x,y,z, qx,qy,qz,qw, type, ...]
    pub fn get_render_data(&self) -> Vec<f32> {
        let rigid_count = self.rigid_body_set.len();
        
        let mut data = Vec::with_capacity(rigid_count * 8);
        
        // Rigid bodies
        for (handle, body) in self.rigid_body_set.iter() {
            let pos = body.translation();
            let rot = body.rotation();
            let obj_type = self.object_types.get(&handle).copied().unwrap_or(0);
            
            data.push(pos.x);
            data.push(pos.y);
            data.push(pos.z);
            data.push(rot.i);
            data.push(rot.j);
            data.push(rot.k);
            data.push(rot.w);
            data.push(obj_type as f32);
        }
        
        // Fluid Particles (type 2)
        for (_handle, fluid) in self.fluid_pipeline.liquid_world.fluids().iter() {
            for particle in &fluid.positions {
                data.push(particle.x);
                data.push(particle.y);
                data.push(particle.z);
                data.push(0.0); // qx
                data.push(0.0); // qy
                data.push(0.0); // qz
                data.push(1.0); // qw
                data.push(2.0); // Type 2 = Liquid
            }
        }
        
        data
    }
}
