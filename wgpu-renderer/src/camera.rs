use cgmath::{Matrix4, Point3, Vector3};
use winit::keyboard::KeyCode;

#[derive(Clone, Debug)]
pub struct CameraController {
    speed: f32,
    sensitivity: f32,
    // 添加键盘/鼠标状态
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn handle_key(&mut self, keycode: KeyCode, is_pressed: bool) -> bool {

        match keycode {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            _ => false
        }
    }

    pub fn update_camera(&mut self, eye: &mut Point3<f32>,
                         target: Point3<f32>,
                         up: Vector3<f32>,) {
        use cgmath::InnerSpace;

        let forward = target - *eye;
        let forward_norm = forward.normalize();// 归一化
        let forward_mag = forward_norm.magnitude();// 分量

        // 向前移动
        if self.is_forward_pressed && forward_mag > self.speed {
            *eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            *eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(up);// 得到向右的向量

        let forward = target - *eye;
        let forward_mag = forward.magnitude();

        if self.is_left_pressed {
            // 需要增加向前+向左的向量
            *eye = target - (forward - right * self.speed).normalize() * forward_mag;
        }
        if self.is_right_pressed {
            // 每次操作变量都是基于target进行向量操作
            *eye = target - (forward + right *self.speed).normalize() * forward_mag;
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniforms {
    // 传递给shader使用的uniform
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 3],
    _padding: f32,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Camera {
    // 实际代码使用的相机对象
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>, // 摄像机的方向,固定
    fov: f32,
    aspect: f32,
    near: f32,
    far: f32,

    // GPU
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,

    pub controller: CameraController,
}

impl Camera {
    pub fn new(device: &wgpu::Device, aspect: f32) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[
                // 整个camera对象
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera uniform_buffer"),
            size: std::mem::size_of::<CameraUniforms>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let controller = CameraController::new(0.1, 0.5);

        Self {
            eye: Point3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::unit_y(),
            fov: 45.0,
            aspect,
            near: 0.01,
            far: 100.0,

            bind_group_layout,
            bind_group,
            uniform_buffer,
            controller,
        }
    }

    // 获取观察矩阵
    pub fn get_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.eye, self.target, self.up)
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        // 投影矩阵
        let proj = cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.near, self.far);

        proj * self.get_view_matrix()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, delta_time: f32) {
        self.controller.update_camera(&mut self.eye, self.target, self.up);
        let camera_uniforms = CameraUniforms{
            view_proj: self.get_projection_matrix().into(),  // 使用投影矩阵 * 视图矩阵
            view_position: self.eye.into(),
            _padding: 0.0,
        };

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&camera_uniforms));
    }
}
