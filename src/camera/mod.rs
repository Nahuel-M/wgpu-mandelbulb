use bytemuck::{Pod, Zeroable};
use ndarray::array;
use std::{f32::consts::PI};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingType, Buffer,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Device, Queue, ShaderStages,
};
use winit::{dpi::PhysicalSize, event::{VirtualKeyCode, KeyboardInput}};

use self::movement::Movement;
pub mod movement;

#[repr(C)]
#[derive(Copy, Clone, Debug, Zeroable, Pod)]
struct CameraUniform {
    ray_dir_mat: [[f32; 4]; 3],
    position: [f32; 3],
    ratio: f32,
    depth: f32,
    _pad: [f32; 3],
}

pub struct CameraManager {
    size: PhysicalSize<u32>,

    pub position: ndarray::Array1<f32>,
    yaw: f32,
    pitch: f32,

    screen_depth: f32,

    camera_uniform: Buffer,
    camera_bind_group: BindGroup,
    movement: Movement,
}

impl CameraManager {
    pub fn new(device: &Device, size: PhysicalSize<u32>) -> Self {
        let camera_uniform = Self::init_buffers(device);
        let bind_group_layout = Self::bind_group_layout(device);

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_uniform.as_entire_binding(),
            }],
        });

        let movement = Movement{speed: 0.001, ..Default::default()};

        Self {
            size,
            position: array![-1.5, 0.0, 0.],
            yaw: 0.0,
            pitch: 0.0,
            screen_depth: 2.0,
            camera_uniform,
            camera_bind_group,
            movement
        }
    }

    pub fn set_size(&mut self, size: PhysicalSize<u32>) {
        self.size = size;
    }

    pub fn bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
        let bind_group_layout = wgpu::BindGroupLayoutDescriptor {
            label: Some("ShapesBindGroupLayout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(std::mem::size_of::<CameraUniform>() as u64),
                },
                count: None,
            }],
        };
        device.create_bind_group_layout(&bind_group_layout)
    }

    fn init_buffers(device: &Device) -> wgpu::Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("Camera Uniform"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn update_buffers(&mut self, queue: &Queue) {
        self.update_camera();
        queue.write_buffer(
            &self.camera_uniform,
            0,
            bytemuck::bytes_of(&self.generate_uniform()),
        );
    }

    fn generate_uniform(&self) -> CameraUniform {
        let forward = self.forward();
        let up = self.up();
        let ratio = self.aspect_ratio();
        let left = -self.right();
        CameraUniform {
            ray_dir_mat: [
                [left[0], left[1], left[2], 0.0],
                [up[0], up[1], up[2], 0.0],
                [forward[0], forward[1], forward[2], 0.0],
            ],
            position: [self.position[0], self.position[1], self.position[2]],
            ratio,
            depth: self.screen_depth,
            _pad: [0.0, 0.0, 0.0],
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.camera_bind_group
    }

    pub fn forward(&self) -> ndarray::Array1<f32> {
        ndarray::arr1(&[self.yaw.cos()*self.pitch.cos(), self.pitch.sin(), self.yaw.sin()*self.pitch.cos()])
    }

    pub fn right(&self) -> ndarray::Array1<f32> {
        ndarray::arr1(&[
            (self.yaw - PI / 2.0).cos(),
            0.0,
            (self.yaw - PI / 2.0).sin(),
        ])
    }

    pub fn up(&self) -> ndarray::Array1<f32> {
        ndarray::arr1(&[-self.yaw.cos()*self.pitch.sin(), self.pitch.cos(), -self.yaw.sin()*self.pitch.sin()])
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    pub fn screen_depth(&self) -> f32 {
        self.screen_depth
    }

    pub fn set_screen_depth(&mut self, depth: f32) {
        self.screen_depth = depth;
    }

    pub fn set_position(&mut self, pos: ndarray::Array1<f32>) {
        if pos.dim() == 3 {
            self.position = pos;
        } else {
            panic!("Expected position to be a vector of length 3");
        }
    }

    pub fn update_camera(&mut self) {
        let movement = &self.movement;
        self.position = &self.position + self.right() * movement.right * movement.speed;
        self.position = &self.position + self.up() * movement.up * movement.speed;
        self.position = &self.position + self.forward() * movement.forward * movement.speed;
        self.pitch += movement.pitch;
        self.yaw += movement.yaw;
        if  self.pitch > PI / 2.{
            self.pitch = PI / 2.
        } else if self.pitch < -PI / 2. {
            self.pitch = -PI / 2.
        }
        if self.yaw > PI * 2. {
            self.yaw -= PI * 2.;
        } else if self.yaw <= 0. {
            self.yaw += PI * 2.;
        }
        self.movement.pitch = 0.;
        self.movement.yaw = 0.;
    }

    pub fn handle_keyboard_input(&mut self, keyboard_input: &KeyboardInput) {
        use VirtualKeyCode::*;
        match keyboard_input {
            KeyboardInput {state, virtual_keycode: Some(A),..} => {
                self.movement.right = 1. - *state as i8 as f32;
            },
            KeyboardInput {state, virtual_keycode: Some(D),..} => {
                self.movement.right = -(1. - *state as i8 as f32);
            },
            KeyboardInput {state, virtual_keycode: Some(S),..} => {
                self.movement.forward = -(1. - *state as i8 as f32);
            },
            KeyboardInput {state, virtual_keycode: Some(W),..} => {
                self.movement.forward = 1. - *state as i8 as f32;
            },
            KeyboardInput {state, virtual_keycode: Some(LShift),..} => {
                self.movement.up = 1. - *state as i8 as f32;
            },
            KeyboardInput {state, virtual_keycode: Some(C),..} => {
                self.movement.up = -(1. - *state as i8 as f32);
            },
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_motion(&mut self, delta: (f64, f64)){
        self.movement.yaw = delta.0 as f32 / 400.;
        self.movement.pitch = -delta.1 as f32 / 400.;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.movement.speed = speed;
    }
}
