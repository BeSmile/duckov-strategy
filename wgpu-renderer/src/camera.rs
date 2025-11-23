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
    // 向上
    is_space_pressed: bool,
    is_shift_left_pressed: bool,
    // 缩放相关
    zoom_delta: f32,
    zoom_speed: f32,
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
            is_space_pressed: false,
            is_shift_left_pressed: false,
            zoom_delta: 0.0,
            zoom_speed: 0.1, // 缩放速度
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
            KeyCode::Space=> {
                self.is_space_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft=> {
                self.is_shift_left_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    /// 处理鼠标滚轮缩放事件
    /// delta > 0: 放大 (向前滚动/双指向上滑动)
    /// delta < 0: 缩小 (向后滚动/双指向下滑动)
    pub fn handle_scroll(&mut self, delta: f32) {
        // 累积滚轮增量
        self.zoom_delta += delta * self.zoom_speed;
    }

    pub fn update_camera(&mut self, eye: &mut Point3<f32>, target: &mut Point3<f32>, up: Vector3<f32>) {
        use cgmath::InnerSpace;

        let forward = *target - *eye;
        let forward_norm = forward.normalize(); // 归一化
        // let forward_mag = forward_norm.magnitude(); // 分量

        let speed = if self.is_shift_left_pressed {
            self.speed * 1.5
        } else {
            self.speed
        };
        // 向前移动
        if self.is_forward_pressed
        // && forward_mag > self.speed
        {
            *eye += forward_norm * speed;
            *target += forward_norm * speed;
        }
        if self.is_backward_pressed {
            *eye -= forward_norm * speed;
            *target -= forward_norm * speed;
        }
        if self.is_space_pressed {
            eye.y += speed;
            target.y += speed;
        }

        let right = forward_norm.cross(up); // 得到向右的向量

        let forward = *target - *eye;
        let forward_mag = forward.magnitude();

        if self.is_left_pressed {
            // 需要增加向前+向左的向量
            *eye = *target - (forward - right * speed).normalize() * forward_mag;
        }
        if self.is_right_pressed {
            // 每次操作变量都是基于target进行向量操作
            *eye = *target - (forward + right * speed).normalize() * forward_mag;
        }

        // 处理缩放：调整相机到目标点的距离
        if self.zoom_delta.abs() > 0.001 {
            let forward = *target - *eye;
            let forward_norm = forward.normalize();
            let mut new_distance = forward.magnitude() - self.zoom_delta;

            // 限制最小和最大缩放距离
            new_distance = new_distance.max(0.5).min(500.0);

            // 更新相机位置，保持朝向target
            *eye = *target - forward_norm * new_distance;

            // 重置缩放增量
            self.zoom_delta = 0.0;
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

        let controller = CameraController::new(0.5, 0.5);

        // 288.4272,
        // 10.784296,
        // 30.982054,
        Self {
            eye: Point3 {
                x: 288.4272,
                y: 10.784296,
                z: 30.982054,
            },
            target: Point3::new(290.4272, 11.784296, 28.982054),
            up: Vector3::unit_y(),
            fov: 45.0,
            aspect,
            near: 0.01,
            far: 1000.0,

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

    // 获取纯投影矩阵（用于射线投射等）
    pub fn get_projection_only(&self) -> Matrix4<f32> {
        cgmath::perspective(cgmath::Deg(self.fov), self.aspect, self.near, self.far)
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        // 投影矩阵 * 视图矩阵（用于shader）
        self.get_projection_only() * self.get_view_matrix()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, delta_time: f32) {
        self.controller
            .update_camera(&mut self.eye, &mut self.target, self.up);
        let camera_uniforms = CameraUniforms {
            view_proj: self.get_projection_matrix().into(), // 使用投影矩阵 * 视图矩阵
            view_position: self.eye.into(),
            _padding: 0.0,
        };
        // println!("camera view_position: {:#?}", self.eye);

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&camera_uniforms),
        );
    }
}
