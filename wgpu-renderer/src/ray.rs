use cgmath::{InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4};

#[derive(Debug)]
pub struct Ray{
    // 所在
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    /// Create a ray from screen coordinates (mouse position)
    /// screen_pos: mouse position in pixels
    /// screen_size: window size in pixels
    /// view_matrix: camera view matrix
    /// projection_matrix: camera projection matrix
    pub fn from_screen_coords(
        screen_pos: (f32, f32),
        screen_size: (u32, u32),
        view_matrix: Matrix4<f32>,
        projection_matrix: Matrix4<f32>,
    ) -> Self {

        // Convert screen coordinates to NDC (-1 to 1)
        let ndc_x = (2.0 * screen_pos.0) / screen_size.0 as f32 - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.1) / screen_size.1 as f32; // Y is flipped

        // Create clip space coordinates for near and far planes
        let clip_near = Vector4::new(ndc_x, ndc_y, 0.0, 1.0); // Near plane in wgpu is 0
        let clip_far = Vector4::new(ndc_x, ndc_y, 1.0, 1.0);  // Far plane is 1

        // Inverse view-projection matrix
        let inv_view_proj = (projection_matrix * view_matrix)
            .invert()
            .unwrap_or(Matrix4::identity());

        // Unproject to world space
        let world_near = inv_view_proj * clip_near;
        let world_far = inv_view_proj * clip_far;

        // Perspective divide
        let near_point = Point3::new(
            world_near.x / world_near.w,
            world_near.y / world_near.w,
            world_near.z / world_near.w,
        );
        let far_point = Point3::new(
            world_far.x / world_far.w,
            world_far.y / world_far.w,
            world_far.z / world_far.w,
        );

        let direction = (far_point - near_point).normalize();

        Ray {
            origin: near_point,
            direction,
        }
    }

    pub fn intersect_aabb(&self, min: Point3<f32>, max: Point3<f32>) -> Option<f32> {
        let inv_dir = Vector3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (min.x - self.origin.x) * inv_dir.x;
        let t2 = (max.x - self.origin.x) * inv_dir.x;
        let t3 = (min.y - self.origin.y) * inv_dir.y;
        let t4 = (max.y - self.origin.y) * inv_dir.y;
        let t5 = (min.z - self.origin.z) * inv_dir.z;
        let t6 = (max.z - self.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            None
        } else {
            Some(if tmin < 0.0 { tmax } else { tmin })
        }
    }

    /// Test intersection with a sphere
    pub fn intersect_sphere(&self, center: Point3<f32>, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t > 0.0 {
                Some(t)
            } else {
                let t = (-b + discriminant.sqrt()) / (2.0 * a);
                if t > 0.0 { Some(t) } else { None }
            }
        }
    }
}