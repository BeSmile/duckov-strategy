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
    // FPS相机控制
    yaw: f32,   // 水平旋转角度（弧度）
    pitch: f32, // 垂直旋转角度（弧度）
    is_mouse_captured: bool, // 是否捕获鼠标进行FPS控制
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
            zoom_speed: 0.4, // 缩放速度
            yaw: 0.0,
            pitch: 0.0,
            is_mouse_captured: false,
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

    /// 处理鼠标移动事件（FPS相机控制）
    /// delta_x, delta_y: 鼠标移动的像素增量
    pub fn handle_mouse_move(&mut self, delta_x: f32, delta_y: f32) {
        if !self.is_mouse_captured {
            return;
        }

        // 更新yaw（水平旋转）和pitch（垂直旋转）
        self.yaw += delta_x * self.sensitivity * 0.001;
        self.pitch -= delta_y * self.sensitivity * 0.001;

        // 限制pitch角度，防止翻转
        let max_pitch = std::f32::consts::FRAC_PI_2 - 0.01; // 89度
        self.pitch = self.pitch.clamp(-max_pitch, max_pitch);
    }

    /// 切换鼠标捕获状态
    pub fn toggle_mouse_capture(&mut self) {
        self.is_mouse_captured = !self.is_mouse_captured;
    }

    /// 设置鼠标捕获状态
    pub fn set_mouse_capture(&mut self, captured: bool) {
        self.is_mouse_captured = captured;
    }

    /// 获取鼠标捕获状态
    pub fn is_mouse_captured(&self) -> bool {
        self.is_mouse_captured
    }

    /// 从当前的eye和target计算初始yaw和pitch
    pub fn init_angles_from_target(&mut self, eye: &Point3<f32>, target: &Point3<f32>) {
        use cgmath::InnerSpace;

        let forward = *target - *eye;

        // 计算yaw（水平角度）
        self.yaw = forward.z.atan2(forward.x);

        // 计算pitch（垂直角度）
        let horizontal_distance = (forward.x * forward.x + forward.z * forward.z).sqrt();
        self.pitch = forward.y.atan2(horizontal_distance);
    }

    pub fn update_camera(&mut self, eye: &mut Point3<f32>, target: &mut Point3<f32>, up: Vector3<f32>) {
        use cgmath::InnerSpace;

        let speed = if self.is_shift_left_pressed {
            self.speed * 1.5
        } else {
            self.speed
        };

        // FPS模式：根据yaw和pitch计算朝向方向
        if self.is_mouse_captured {
            // 根据yaw和pitch计算前向向量
            let forward_dir = Vector3::new(
                self.yaw.cos() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.sin() * self.pitch.cos(),
            );

            // 计算右向向量（用于左右移动）
            let right = forward_dir.cross(up).normalize();

            // 计算前向向量的水平分量（用于WASD移动，不包含垂直分量）
            let forward_horizontal = Vector3::new(
                self.yaw.cos(),
                0.0,
                self.yaw.sin(),
            ).normalize();

            // 处理WASD移动
            if self.is_forward_pressed {
                *eye += forward_horizontal * speed;
            }
            if self.is_backward_pressed {
                *eye -= forward_horizontal * speed;
            }
            if self.is_left_pressed {
                *eye -= right * speed;
            }
            if self.is_right_pressed {
                *eye += right * speed;
            }
            if self.is_space_pressed {
                eye.y += speed;
            }

            // 更新target位置（在eye前方固定距离）
            let look_distance = 10.0; // 视线距离
            *target = *eye + forward_dir * look_distance;

        } else {
            // 非FPS模式：保持原有的相机控制逻辑
            let forward = *target - *eye;
            let forward_norm = forward.normalize();

            // 向前移动
            if self.is_forward_pressed {
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

            let right = forward_norm.cross(up);
            let forward = *target - *eye;
            let forward_mag = forward.magnitude();

            if self.is_left_pressed {
                *eye = *target - (forward - right * speed).normalize() * forward_mag;
            }
            if self.is_right_pressed {
                *eye = *target - (forward + right * speed).normalize() * forward_mag;
            }

            // 处理缩放：调整相机到目标点的距离
            if self.zoom_delta.abs() > 0.001 {
                let forward = *target - *eye;
                let forward_norm = forward.normalize();
                let new_distance = (forward.magnitude() - self.zoom_delta).clamp(0.1, 1000.0);

                *eye = *target - forward_norm * new_distance;
                self.zoom_delta = 0.0;
            }
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
    
    pub fn set_target(&mut self, target: Point3<f32>) {
        self.target = target;
    }

    pub fn set_eye(&mut self, eye: Point3<f32>) {
        self.eye = eye;
    }

    pub fn eye(&self) -> &Point3<f32> {
        &self.eye
    }

    pub fn target(&self) -> &Point3<f32> {
        &self.target
    }

    pub fn update(&mut self, queue: &wgpu::Queue, delta_time: f32) {
        self.controller
            .update_camera(&mut self.eye, &mut self.target, self.up);
        let camera_uniforms = CameraUniforms {
            view_proj: self.get_projection_matrix().into(), // 使用投影矩阵 * 视图矩阵
            view_position: self.eye.into(),
            _padding: 0.0,
        };

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&camera_uniforms),
        );
    }
}
