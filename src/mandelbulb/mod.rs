use bytemuck::{Zeroable, Pod};
use wgpu::{BindGroupLayoutEntry, BindingType, ShaderStages, BufferBindingType, BufferSize, Device, BufferDescriptor, BufferUsages, Queue, BindGroupEntry, BindGroupDescriptor, Buffer, BindGroup};

#[repr(C, align(8))]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
pub struct MandelBulbUniform{
    pub iterations: i32,
    pub max_ray_march_iterations: i32,
    pub collision_distance: f32,

    pub power : f32,

    pub color_map_black: [f32; 4],
    pub color_map_white: [f32; 4],
}
pub struct MandelbulbManager{
    pub mandelbulb: MandelBulbUniform,
    mandelbulb_uniform: Buffer,
    mandelbulb_bind_group: BindGroup,
}

impl MandelbulbManager{
    pub fn new(device: &Device) -> Self {
        let mandelbulb_uniform = Self::init_buffers(device);
        let bind_group_layout = Self::bind_group_layout(device);

        let mandelbulb_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("MandelbulbBindGroup"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: mandelbulb_uniform.as_entire_binding(),
            }],
        });

        let mandelbulb = MandelBulbUniform{
            iterations: 20,
            max_ray_march_iterations: 200,
            collision_distance: 0.0001,
            power: 7.0,
            color_map_black: [0.1, 0.1, 0.05, 0.], 
            color_map_white: [0.8, 0.3, 0.1, 0.],
        };
        Self {
            mandelbulb,
            mandelbulb_uniform,
            mandelbulb_bind_group,
        }
    }

    pub fn bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
        let bind_group_layout = wgpu::BindGroupLayoutDescriptor {
            label: Some("MandulbulbBindGroupLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(std::mem::size_of::<MandelBulbUniform>() as u64),
                },
                count: None,
            }],
        };
        device.create_bind_group_layout(&bind_group_layout)
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.mandelbulb_bind_group
    }

    fn init_buffers(device: &Device) -> wgpu::Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("Mandelbulb Uniform"),
            size: std::mem::size_of::<MandelBulbUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn update_buffers(&self, queue: &Queue) {
        queue.write_buffer(
            &self.mandelbulb_uniform,
            0,
            bytemuck::bytes_of(&self.mandelbulb),
        );
    }
}

impl MandelBulbUniform{
    pub fn estimated_distance(&self, position: &[f32]) -> f32{
        let mut z = [position[0], position[1], position[2]];
        let mut dr : f32 = 1.0;
        let mut r : f32 = 0.0;
        for _ in 0..self.iterations {
            r = length(z);
            if r > 2. {
                break;
            }
    
            // convert to polar coordinates
            let mut theta : f32 = f32::acos(z[2]/r);
            let mut phi : f32 = f32::atan2(z[1],z[0]);
            dr = r.powf(self.power - 1.0)*self.power*dr + 1.0;
            
            // scale and rotate the point
            let zr = r.powf(self.power);
            theta *= self.power;
            phi *= self.power;
            
            // convert back to cartesian coordinates
            z = [zr*theta.sin()*phi.cos() + position[0], zr*phi.sin()*theta.sin() + position[1], zr*theta.cos() + position[2]];
        }
        0.5*r.ln()*r/dr
    }
}

fn length(vec3: [f32;3]) -> f32{
    (vec3[0]*vec3[0] + vec3[1]*vec3[1] + vec3[2]*vec3[2]).sqrt()
}