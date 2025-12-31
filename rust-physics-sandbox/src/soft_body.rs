use nalgebra::{Vector3, Point3};

#[derive(Clone, Copy)]
pub struct Particle {
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub mass: f32,
    pub inv_mass: f32,
    pub pinned: bool,
}

impl Particle {
    pub fn new(x: f32, y: f32, z: f32, mass: f32) -> Self {
        Self {
            position: Point3::new(x, y, z),
            velocity: Vector3::zeros(),
            mass,
            inv_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            pinned: false,
        }
    }
}

pub struct Spring {
    pub p1: usize,
    pub p2: usize,
    pub rest_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

pub struct SoftBody {
    pub particles: Vec<Particle>,
    pub springs: Vec<Spring>,
}

impl SoftBody {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            springs: Vec::new(),
        }
    }

    pub fn create_cloth(&mut self, width: usize, height: usize, spacing: f32, offset: Vector3<f32>) {
        let start_idx = self.particles.len();
        
        // Create particles
        for z in 0..height {
            for x in 0..width {
                let mut p = Particle::new(
                    offset.x + x as f32 * spacing,
                    offset.y,
                    offset.z + z as f32 * spacing,
                    1.0 // mass
                );
                
                // Pin two corners
                if z == 0 && (x == 0 || x == width - 1) {
                    p.pinned = true;
                    p.inv_mass = 0.0;
                }
                
                self.particles.push(p);
            }
        }

        // Create springs (structural and shear)
        for z in 0..height {
            for x in 0..width {
                let idx = start_idx + z * width + x;
                
                // Structural right
                if x < width - 1 {
                    self.add_spring(idx, idx + 1, spacing, 100.0, 0.5);
                }
                // Structural down
                if z < height - 1 {
                    self.add_spring(idx, idx + width, spacing, 100.0, 0.5);
                }
                
                // Shearing (diagonal) could be added for better stability
            }
        }
    }

    fn add_spring(&mut self, p1: usize, p2: usize, rest_length: f32, stiffness: f32, damping: f32) {
        self.springs.push(Spring {
            p1,
            p2,
            rest_length,
            stiffness,
            damping,
        });
    }

    pub fn step(&mut self, dt: f32, gravity: &Vector3<f32>) {
        // 1. Apply gravity and external forces
        for p in &mut self.particles {
            if p.pinned { continue; }
            // F = ma -> a = F/m. gravity is acceleration.
            p.velocity += gravity * dt;
        }

        // 2. Solve springs (relaxation)
        // We do a few iterations for stiffness
        let iterations = 2;
        for _ in 0..iterations {
            for spring in &self.springs {
                let p1_idx = spring.p1;
                let p2_idx = spring.p2;
                
                // We need to borrow particles independently. 
                // Unsafe or index based approach needed to get two mut refs from vec.
                // Or just use indexing and copy values, then update.
                
                let p1 = self.particles[p1_idx];
                let p2 = self.particles[p2_idx];
                
                let delta = p2.position - p1.position;
                let dist = delta.norm();
                
                if dist < 1e-4 { continue; } // Avoid division by zero
                
                let diff = (dist - spring.rest_length) / dist;
                let correction = delta * 0.5 * diff * 0.8; // 0.8 is roughly stiffness factor for PBD
                
                if !p1.pinned {
                    self.particles[p1_idx].position += correction;
                    // Simple damping
                    self.particles[p1_idx].velocity *= 0.999;
                }
                if !p2.pinned {
                    self.particles[p2_idx].position -= correction;
                    self.particles[p2_idx].velocity *= 0.999;
                }
            }
        }

        // 3. Integrate position
        for p in &mut self.particles {
            if p.pinned { continue; }
            p.position += p.velocity * dt;
            
            // Floor collision (simple)
            if p.position.y < 0.0 {
                p.position.y = 0.0;
                p.velocity.y = 0.0;
                p.velocity.x *= 0.9; // Friction
                p.velocity.z *= 0.9;
            }
        }
    }
}
