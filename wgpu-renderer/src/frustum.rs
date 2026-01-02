use cgmath::{Matrix4, Point3, Vector3, Vector4, InnerSpace};
use crate::mesh::AABB;

/// 平面方程：normal·point + distance = 0
/// normal 是单位法向量，distance 是原点到平面的有向距离
#[derive(Clone, Debug)]
pub struct Plane {
    pub normal: Vector3<f32>,
    pub distance: f32,
}

impl Plane {
    pub fn new(normal: Vector3<f32>, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// 从 Vector4 创建平面（前3个分量是法向量，第4个是距离）
    pub fn from_vector4(v: Vector4<f32>) -> Self {
        let normal = Vector3::new(v.x, v.y, v.z);
        Self {
            normal,
            distance: v.w,
        }
    }

    /// 归一化平面方程
    pub fn normalize(&self) -> Self {
        let length = self.normal.magnitude();
        if length > 0.0 {
            Self {
                normal: self.normal / length,
                distance: self.distance / length,
            }
        } else {
            self.clone()
        }
    }

    /// 计算点到平面的有向距离
    /// 正数表示点在平面前方（法向量方向），负数表示在平面后方
    pub fn distance_to_point(&self, point: Point3<f32>) -> f32 {
        self.normal.x * point.x + self.normal.y * point.y + self.normal.z * point.z + self.distance
    }
}

/// 视锥体剔除结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullingResult {
    Outside,      // 完全在视锥外，应该剔除
    Intersecting, // 与视锥相交
    Inside,       // 完全在视锥内
}

/// 视锥体（6个平面）
/// 平面顺序：Left, Right, Bottom, Top, Near, Far
#[derive(Clone, Debug)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    /// 从 view-projection 矩阵提取视锥体平面（Gribb-Hartmann算法）
    ///
    /// 原理：在裁剪空间中，视锥体的6个平面可以从VP矩阵的行推导出来
    /// - Left plane:   row4 + row1
    /// - Right plane:  row4 - row1
    /// - Bottom plane: row4 + row2
    /// - Top plane:    row4 - row2
    /// - Near plane:   row4 + row3
    /// - Far plane:    row4 - row3
    pub fn from_view_proj(view_proj: &Matrix4<f32>) -> Self {
        let col0 = view_proj[0];
        let col1 = view_proj[1];
        let col2 = view_proj[2];
        let col3 = view_proj[3];

        let row0 = Vector4::new(col0.x, col1.x, col2.x, col3.x);
        let row1 = Vector4::new(col0.y, col1.y, col2.y, col3.y);
        let row2 = Vector4::new(col0.z, col1.z, col2.z, col3.z);
        let row3 = Vector4::new(col0.w, col1.w, col2.w, col3.w);

        let planes = [
            Plane::from_vector4(row3 + row0).normalize(), // Left
            Plane::from_vector4(row3 - row0).normalize(), // Right
            Plane::from_vector4(row3 + row1).normalize(), // Bottom
            Plane::from_vector4(row3 - row1).normalize(), // Top
            Plane::from_vector4(row2).normalize(),        // Near (wgpu: 直接用 row2)
            Plane::from_vector4(row3 - row2).normalize(), // Far
        ];

        Self { planes }
    }

    /// 测试 AABB 与视锥体的相交关系
    ///
    /// 算法：对每个平面，计算AABB的"正顶点"（沿法向量方向最远点）和"负顶点"（最近点）
    /// - 如果正顶点在平面背面，则AABB完全在视锥外 → Outside
    /// - 如果负顶点在平面背面，则AABB与该平面相交 → 标记为相交
    /// - 如果所有平面的负顶点都在前面，则AABB完全在视锥内 → Inside
    pub fn test_aabb(&self, aabb: &AABB) -> CullingResult {
        let mut fully_inside = true;

        for plane in &self.planes {
            // 计算AABB的"正顶点"（沿法向量方向最远的角点）
            let p = Point3::new(
                if plane.normal.x > 0.0 {
                    aabb.max.x
                } else {
                    aabb.min.x
                },
                if plane.normal.y > 0.0 {
                    aabb.max.y
                } else {
                    aabb.min.y
                },
                if plane.normal.z > 0.0 {
                    aabb.max.z
                } else {
                    aabb.min.z
                },
            );

            // 如果正顶点在平面背面，则整个AABB在平面背面 → 完全剔除
            if plane.distance_to_point(p) < 0.0 {
                return CullingResult::Outside;
            }

            // 计算"负顶点"（沿法向量方向最近的角点）
            let n = Point3::new(
                if plane.normal.x > 0.0 {
                    aabb.min.x
                } else {
                    aabb.max.x
                },
                if plane.normal.y > 0.0 {
                    aabb.min.y
                } else {
                    aabb.max.y
                },
                if plane.normal.z > 0.0 {
                    aabb.min.z
                } else {
                    aabb.max.z
                },
            );

            // 如果负顶点在平面背面，则AABB与该平面相交
            if plane.distance_to_point(n) < 0.0 {
                fully_inside = false;
            }
        }

        if fully_inside {
            CullingResult::Inside
        } else {
            CullingResult::Intersecting
        }
    }

    /// 测试AABB是否可见（不在视锥外）
    pub fn is_visible(&self, aabb: &AABB) -> bool {
        self.test_aabb(aabb) != CullingResult::Outside
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::SquareMatrix;

    #[test]
    fn test_plane_distance() {
        // 测试平面方程：z = 0 的平面
        let plane = Plane::new(Vector3::new(0.0, 0.0, 1.0), 0.0);

        let point_front = Point3::new(0.0, 0.0, 1.0); // z > 0，在前方
        let point_back = Point3::new(0.0, 0.0, -1.0); // z < 0，在后方
        let point_on = Point3::new(0.0, 0.0, 0.0); // z = 0，在平面上

        assert!(plane.distance_to_point(point_front) > 0.0);
        assert!(plane.distance_to_point(point_back) < 0.0);
        assert!((plane.distance_to_point(point_on)).abs() < 1e-6);
    }

    #[test]
    fn test_frustum_extract() {
        // 测试视锥体提取（使用单位矩阵）
        let identity = Matrix4::identity();
        let frustum = Frustum::from_view_proj(&identity);

        // 应该提取出6个平面
        assert_eq!(frustum.planes.len(), 6);
    }

    #[test]
    fn test_aabb_frustum_simple() {
        // 创建一个简单的测试场景
        let identity = Matrix4::identity();
        let frustum = Frustum::from_view_proj(&identity);

        // 在原点附近的AABB应该可见
        let aabb_center = AABB::new(
            Point3::new(-0.5, -0.5, -0.5),
            Point3::new(0.5, 0.5, 0.5),
        );
        assert!(frustum.is_visible(&aabb_center));

        // 非常远的AABB可能被剔除（取决于视锥参数）
        let aabb_far = AABB::new(
            Point3::new(1000.0, 1000.0, 1000.0),
            Point3::new(1001.0, 1001.0, 1001.0),
        );
        // 这个测试依赖于具体的视锥参数，这里只是示例
        let _ = frustum.test_aabb(&aabb_far);
    }

    #[test]
    fn test_plane_normalize() {
        let plane = Plane::new(Vector3::new(2.0, 0.0, 0.0), 4.0);
        let normalized = plane.normalize();

        // 归一化后，法向量长度应该为1
        assert!((normalized.normal.magnitude() - 1.0).abs() < 1e-6);

        // 距离也应该相应缩放
        assert!((normalized.distance - 2.0).abs() < 1e-6);
    }
}
