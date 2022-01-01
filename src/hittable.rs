use crate::render::SceneData;
use crate::utility::*;
use crate::bvh::*;
use crate::mesh::*;
use crate::material::MaterialId;

// ------------------------------------------- Hittable -------------------------------------------

#[derive(Clone)]
pub enum Hittable {
    Sphere {center: Rvec3, radius: Real, material: MaterialId},
    Triangle {triangle: TriangleId, mesh: MeshId},
    List(Vec<Hittable>),
    Bvh(Bvh),
}

impl Hittable {
    pub fn hit(&self, ray: &Ray, scene_data: &SceneData) -> Option<(Hit, MaterialId)> {
        match self {
            Self::Sphere {center, radius, material} => hit_sphere(center, *radius, *material, ray),
            Self::Triangle {triangle, mesh} => hit_triangle(*triangle, *mesh, ray, scene_data),
            Self::List(list) => hit_list(list, ray, scene_data),
            Self::Bvh(bvh) => bvh.hit(ray, scene_data),
        }
    }

    pub fn bounding_box(&self, scene_data: &SceneData) -> AABB {
        match self {
            Self::Sphere {center, radius, ..} => bounding_box_sphere(center, *radius),
            Self::Triangle {triangle, mesh} => bounding_box_triangle(*triangle, *mesh, scene_data),
            Self::List(list) => bounding_box_list(list, scene_data),
            Self::Bvh(_) => panic!("Do not take the bounding box of a Bvh. What are you trying to do?")
        }
    }
}

// ------------------------------------------- Hit implementations -------------------------------------------

fn hit_sphere(center: &Rvec3, radius: Real, material: MaterialId, ray: &Ray) -> Option<(Hit, MaterialId)> {
    let to_center = ray.origin - center;
    let a = ray.direction.norm_squared();
    let half_b = ray.direction.dot(&to_center);
    let c = to_center.norm_squared() - radius*radius;
    let delta = half_b*half_b - a*c;
    if delta <= 0.0 {
        return None
    }
    
    // Compute the intersection parameter t
    let sqrt_delta = delta.sqrt();
    let mut t = (-half_b - sqrt_delta) / a; // Try the closer hit
    if t < ray.t_min || t > ray.t_max {
        t = (-half_b + sqrt_delta) / a; // Try the further hit
        if t < ray.t_min || t > ray.t_max {
            return None
        }
    }

    let position = ray.at(t);
    let normal = (position - center).normalize();
    let uv = vector![0.5 - normal.z.atan2(normal.x) / TAU, normal.y.asin() / PI + 0.5];
    Some((Hit {t, position, normal, uv}, material))
}

fn hit_triangle(triangle: TriangleId, mesh: MeshId, ray: &Ray, scene_data: &SceneData) -> Option<(Hit, MaterialId)> {
    // https://facultyweb.cs.wwu.edu/~wehrwes/courses/csci480_20w/lectures/L10/L10.pdf
    let triangle = scene_data.mesh_table[mesh.to_index()].get_triangle(triangle);
    let a = triangle.0.position;
    let b = triangle.1.position;
    let c = triangle.2.position;
    let ba = a - b;
    let ca = a - c;
    let pa = a - ray.origin;
    let d = ray.direction;

    // Solve this system of equations: [ a-b  a-c  d ] * [ u  v  t ]^T = a-p
    let det = ba.x * ca.y * d.z + ba.y * ca.z * d.x + ba.z * ca.x * d.y
            - ba.x * ca.z * d.y - ba.y * ca.x * d.z - ba.z * ca.y * d.x;

    if det.abs() < SMOL {
        return None
    }
    let inv_det = 1.0 / det;

    let t = (pa.x * (ba.y * ca.z - ba.z * ca.y)
           + pa.y * (ba.z * ca.x - ba.x * ca.z)
           + pa.z * (ba.x * ca.y - ba.y * ca.x)) * inv_det;
    
    let u = (pa.x * (ca.y * d.z - ca.z * d.y)
           + pa.y * (ca.z * d.x - ca.x * d.z)
           + pa.z * (ca.x * d.y - ca.y * d.x)) * inv_det;

    let v = (pa.x * (ba.z * d.y - ba.y * d.z)
           + pa.y * (ba.x * d.z - ba.z * d.x)
           + pa.z * (ba.y * d.x - ba.x * d.y)) * inv_det;
    
    let w = 1.0 - u - v;
        
    if t < ray.t_min || t > ray.t_max || u < 0.0 || v < 0.0 || w < 0.0 {
        return None
    }

    // Interpolate the normals and texture coordinates
    let position = ray.at(t);
    let normal = w * triangle.0.normal + u * triangle.1.normal + v * triangle.2.normal;
    let uv = w * triangle.0.uv + u * triangle.1.uv + v * triangle.2.uv;
    Some((Hit {t, position, normal, uv}, scene_data.mesh_table[mesh.to_index()].material))
}

fn hit_list(list: &[Hittable], ray: &Ray, scene_data: &SceneData) -> Option<(Hit, MaterialId)> {
    let mut hit = None;
    let mut ray = ray.clone();
    for x in list {
        if let Some(new_hit) = x.hit(&ray, scene_data) {
            ray.t_max = new_hit.0.t;
            hit.replace(new_hit);
        }
    }
    hit
}

// ------------------------------------------- Bounding box implementation -------------------------------------------

fn bounding_box_sphere(center: &Rvec3, radius: Real) -> AABB {
    AABB {
        min: center - vector![radius, radius, radius],
        max: center + vector![radius, radius, radius],
    }
}

fn bounding_box_triangle(triangle: TriangleId, mesh: MeshId, scene_data: &SceneData) -> AABB {
    let triangle = scene_data.mesh_table[mesh.to_index()].get_triangle(triangle);
    let a = triangle.0.position;
    let b = triangle.1.position;
    let c = triangle.2.position;
    AABB {
        min: vector![a.x.min(b.x).min(c.x), a.y.min(b.y).min(c.y), a.z.min(b.z).min(c.z)],
        max: vector![a.x.max(b.x).max(c.x), a.y.max(b.y).max(c.y), a.z.max(b.z).max(c.z)],
    }
}

fn bounding_box_list(list: &[Hittable], scene_data: &SceneData) -> AABB {
    if list.is_empty() {
        return AABB::default();
    }
    list.iter().skip(1).fold(list[0].bounding_box(scene_data), |aabb, x| aabb.union(&x.bounding_box(scene_data)))
}
