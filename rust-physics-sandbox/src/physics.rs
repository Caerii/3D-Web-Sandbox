use rapier3d::prelude::*;
use std::collections::HashMap;

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
            object_types: HashMap::new(),
        }
    }

    pub fn step(&mut self) {
        let physics_hooks = ();
        let event_handler = ();

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

    // Returns a flattened list of transforms: [x,y,z, qx,qy,qz,qw, type, ...]
    pub fn get_render_data(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.rigid_body_set.len() * 8);
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
        data
    }
}
