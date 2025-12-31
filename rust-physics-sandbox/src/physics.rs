use rapier3d::prelude::*;
use std::collections::HashMap;
use salva3d::integrations::rapier::FluidsPipeline;
use salva3d::object::{Fluid, FluidHandle};
use nalgebra::Point3;

pub struct PhysicsWorld {
    pub pipeline: PhysicsPipeline,
    pub query_pipeline: QueryPipeline,
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
            query_pipeline: QueryPipeline::new(),
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
        
        // Update query pipeline
        self.query_pipeline.update(&self.rigid_body_set, &self.collider_set);

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

    pub fn cast_ray(&self, origin_x: f32, origin_y: f32, origin_z: f32, dir_x: f32, dir_y: f32, dir_z: f32) -> Option<u32> {
        let ray = Ray::new(
            point![origin_x, origin_y, origin_z],
            vector![dir_x, dir_y, dir_z],
        );
        let max_toi = 100.0;
        let solid = true;
        let query_filter = QueryFilter::default().groups(InteractionGroups::all());

        if let Some((handle, _toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_toi,
            solid,
            query_filter,
        ) {
            // We return the handle as u32. 
            // RigidBodyHandle in Rapier is generational index, but we can just use the index part for simplicity if we trust generation match
            // Or better, we return the index. 
            // Rapier's handles are (index, generation). 
            // For now, let's just return the raw index, assuming we won't have generation conflicts in this simple demo.
            return Some(handle.into_raw_parts().0);
        }
        None
    }
    pub fn apply_impulse(&mut self, handle_idx: u32, x: f32, y: f32, z: f32) {
        // Reconstruct handle (assuming generation 0 or iterating to find match, but for now we try constructing from raw parts)
        // Rapier handle is (index, generation). We guess generation 0.
        // A safer way is to store handles in a map, but we don't have that map inverse.
        // Let's iterate and find the body with this index.
        
        // Actually, rigid_body_set.get_mut takes a RigidBodyHandle.
        // We need to know the generation.
        // Hack: Assume we passed the raw parts correctly or find it.
        
        // Better approach: Iterate and match index.
        let mut target_handle = None;
        for (h, _b) in self.rigid_body_set.iter() {
            if h.into_raw_parts().0 == handle_idx {
                target_handle = Some(h);
                break;
            }
        }
        
        if let Some(h) = target_handle {
            if let Some(body) = self.rigid_body_set.get_mut(h) {
                body.apply_impulse(vector![x, y, z], true);
            }
        }
    }

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
