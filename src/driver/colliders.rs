use std::ops::{Add, AddAssign};

use hord3::horde::geometry::{Intersection, rotation::{Orientation, Rotation}, shapes_3d::{FixedConvexFace, Sphere, Triangle}, vec3d::{Vec3D, Vec3Df}};
use to_from_bytes_derive::{FromBytes, ToBytes};

use crate::{game_engine::CoolVoxel, game_map::{GameMap, VoxelType, get_voxel_pos, road::Road}};

#[derive(Clone, Copy, Debug, PartialEq, ToBytes, FromBytes)]
pub struct AABB {
    max: Vec3Df,
    min: Vec3Df,
}

impl AABB {
    pub fn get_vertices(&self) -> [Vec3Df; 8] {
        [
            self.max,
            self.min,
            Vec3Df::new(self.max.x, self.min.y, self.min.z),
            Vec3Df::new(self.min.x, self.max.y, self.min.z),
            Vec3Df::new(self.min.x, self.min.y, self.max.z),
            Vec3Df::new(self.max.x, self.max.y, self.min.z),
            Vec3Df::new(self.max.x, self.min.y, self.max.z),
            Vec3Df::new(self.min.x, self.max.y, self.max.z),
        ]
    }
    pub fn get_ground_vertices(&self) -> [Vec3Df ; 4] {
        [
            self.min,
            Vec3Df::new(self.max.x, self.min.y, self.min.z),
            Vec3Df::new(self.max.x, self.max.y, self.min.z),
            Vec3Df::new(self.min.x, self.max.y, self.min.z),
        ]
    }
    pub fn get_top_vertices(&self) -> [Vec3Df ; 4] {
        [   
            Vec3Df::new(self.min.x, self.min.y, self.max.z),
            Vec3Df::new(self.max.x, self.min.y, self.max.z),
            self.max,
            Vec3Df::new(self.min.x, self.max.y, self.max.z),
        ]
    }

    pub fn get_triangles(&self) -> [Triangle ; 12] {
        let top = self.get_top_vertices();
        let bot = self.get_ground_vertices();
        [
            // Top face
            Triangle::new([top[0], top[1], top[2]]),
            Triangle::new([top[0], top[3], top[2]]),

            // bottom face
            Triangle::new([bot[0], bot[1], bot[2]]),
            Triangle::new([bot[0], bot[3], bot[2]]),
            
            // back face
            Triangle::new([bot[0], top[0], top[1]]),
            Triangle::new([bot[1], top[1], bot[0]]),

            // front face
            Triangle::new([bot[3], top[3], top[2]]),
            Triangle::new([bot[2], top[2], bot[3]]),

            // right face
            Triangle::new([bot[1], top[1], top[2]]),
            Triangle::new([bot[2], top[2], bot[1]]),

            // left face
            Triangle::new([bot[3], top[3], top[0]]),
            Triangle::new([bot[0], top[0], bot[3]]),
        ]
    }
    pub fn get_first_point(&self) -> Vec3Df {
        self.min
    }
    pub fn get_second_point(&self) -> Vec3Df {
        self.max
    }
    pub fn get_minimum_side_length(&self) -> f32 {
        let sides = self.max - self.min;
        (sides.x.min(sides.y)).min(sides.z).abs()
    }
    pub fn new_precomputed(min: Vec3Df, max: Vec3Df) -> AABB {
        AABB { max, min }
    }
    pub fn new(point1: Vec3Df, point2: Vec3Df) -> AABB {
        let mut max = Vec3Df::new(0.0, 0.0, 0.0);
        let mut min = Vec3Df::new(0.0, 0.0, 0.0);
        if point1.x > point2.x {
            max.x = point1.x;
            min.x = point2.x;
        } else {
            max.x = point2.x;
            min.x = point1.x;
        }
        if point1.y > point2.y {
            max.y = point1.y;
            min.y = point2.y;
        } else {
            max.y = point2.y;
            min.y = point1.y;
        }
        if point1.z > point2.z {
            max.z = point1.z;
            min.z = point2.z;
        } else {
            max.z = point2.z;
            min.z = point1.z;
        }
        AABB { max, min }
    }
    pub fn get_both_points(&self) -> (Vec3Df, Vec3Df) {
        (self.min, self.max)
    }
    pub fn collision_point(&self, cible: &Vec3Df) -> bool {
        return (cible.x >= self.min.x && cible.x <= self.max.x)
            && (cible.y >= self.min.y && cible.y <= self.max.y)
            && (cible.z >= self.min.z && cible.z <= self.max.z);
    }
    pub fn collision_aabb(&self, cible: &AABB) -> bool {
        return (cible.max.x >= self.min.x && cible.min.x <= self.max.x)
            && (cible.max.y >= self.min.y && cible.min.y <= self.max.y)
            && (cible.max.z >= self.min.z && cible.min.z <= self.max.z);
    }
    pub fn update_avec_spd(&mut self, speed: Vec3Df) {
        self.max.x += speed.x;
        self.max.y += speed.y;
        self.max.z += speed.z;
        self.min.x += speed.x;
        self.min.y += speed.y;
        self.min.z += speed.z;
    }
    pub fn merge_with(&self, rhs:&AABB) -> Self {
        Self::new_precomputed(Vec3Df::new(
            self.min.x.min(rhs.min.x),
            self.min.y.min(rhs.min.y),
            self.min.z.min(rhs.min.z)
        ), Vec3Df::new(
            self.max.x.min(rhs.max.x),
            self.max.y.min(rhs.max.y),
            self.max.z.min(rhs.max.z)
        ))
    }
    pub fn collision_world(&self, world:&GameMap<CoolVoxel, Road>) -> bool {
        for vertex in self.get_vertices() {
            match world.get_voxel_at(get_voxel_pos(vertex)) {
                Some(voxel) => {    
                    if !world.get_voxel_types()[voxel.voxel_type as usize].is_completely_empty() {
                        return true
                    }
                },
                None => {
                    return true
                }
            }
        }
        false
    }

    
    /// Does not work when rotating an AABB with any coordinate greater than 100000.0
    pub fn rotate(&self, rotation:&Rotation) -> AABB {
        
        let mut min = Vec3Df::all_ones() * 100000.0;
        let mut max = -Vec3Df::all_ones() * 100000.0;
        for rotated in rotation.rotate_array(&self.get_vertices()) {
            min.x = min.x.min(rotated.x);
            min.y = min.y.min(rotated.y);
            min.z = min.z.min(rotated.z);
            
            max.x = max.x.max(rotated.x);
            max.y = max.y.max(rotated.y);
            max.z = max.z.max(rotated.z);
        }
        AABB::new_precomputed(min, max)
    }
}

/// https://gamedev.stackexchange.com/questions/156870/how-do-i-implement-a-aabb-sphere-collision
fn aabb_sphere_collision(aabb:&AABB, sphere:&BoundingSphere) -> bool {
    // i is a coordinate here
    // float v = p[i];
    //    if( v < b.min[i] ) v = b.min[i]; // v = max( v, b.min[i] )
    //    if( v > b.max[i] ) v = b.max[i]; // v = min( v, b.max[i] )
    //    q[i] = v;
    let p = sphere.center;
    let mut q = sphere.center;
    if p.x < aabb.min.x {
        q.x = aabb.min.x;
    }
    if p.x > aabb.max.x {
        q.x = aabb.max.x;
    }

    if p.y < aabb.min.y {
        q.y = aabb.min.y;
    }
    if p.y > aabb.max.y {
        q.y = aabb.max.y;
    }

    if p.z < aabb.min.z {
        q.z = aabb.min.z;
    }
    if p.z > aabb.max.z {
        q.z = aabb.max.z;
    }
    return sphere.center.dist(&q) < sphere.radius
}

fn aabb_triangle_collision(aabb:&AABB, triangle:&Triangle) -> bool {
    for tri in aabb.get_triangles() {
        if triangle.intersect_with(&tri) {
            return true
        }
    }
    false
} 

impl Add<Vec3Df> for AABB {
    type Output = AABB;
    fn add(self, rhs: Vec3Df) -> Self::Output {
        AABB::new_precomputed(self.min + rhs, self.max + rhs)
    }
}

impl AddAssign<Vec3Df> for AABB {
    fn add_assign(&mut self, rhs: Vec3Df) {
        self.max += rhs;
        self.min += rhs;
    }
}
#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub struct BoundingSphere {
    center:Vec3Df,
    radius:f32,
}

impl BoundingSphere {
    pub fn new(center:Vec3Df, radius:f32) -> Self {
        Self { center, radius }
    }
    pub fn collides_with_sphere(&self, rhs:&Self) -> bool {
        self.radius + rhs.radius > self.center.dist(&rhs.center)
    }
    pub fn collision_world(&self, world:&GameMap<CoolVoxel, Road>) -> bool {
        // incomplete
        let points = [
            self.center - Vec3Df::new(0.0, 0.0, self.radius)
        ];
        for point in points {
            if world.is_voxel_solid(get_voxel_pos(point)) {
                return  true;
            }
        }
        false
    }
    pub fn point_inside(&self, point:Vec3Df) -> bool {
        self.center.dist(&point) <= self.radius
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct ComplexCollider {
    global_aabb:AABB,
    sub_colliders:Vec<SubCollider>,
}

pub enum Collision {
    No,
    Yes
}

impl ComplexCollider {
    pub fn get_global_aabb(&self) -> &AABB {
        &self.global_aabb
    }
    pub fn set_aabb(&mut self, new_aabb:AABB) {
        self.global_aabb = new_aabb;
    }
    pub fn new(global_aabb:AABB, sub_colliders:Vec<SubCollider>) -> Self {
        Self { global_aabb, sub_colliders }
    }
    pub fn merge_with(&self, rhs:&Self) -> Self {
        let mut out = self.clone();
        out.global_aabb = self.global_aabb.merge_with(&rhs.global_aabb);
        for sub_col in &rhs.sub_colliders {
            out.sub_colliders.push(sub_col.clone());
        }
        out
    }
    pub fn get_collision_with(&self, rhs:&Self) ->  Collision {
        if self.global_aabb.collision_aabb(&rhs.global_aabb) {
            // TODO : Handle cases where there aren't sub colliders on at least 1 side
            self.sub_colliders_collide(rhs)
        }
        else {
            Collision::No
        }
    }
    pub fn sub_colliders_collide(&self, rhs:&Self) -> Collision {
        for i in 0..self.sub_colliders.len() {
            for j in 0..rhs.sub_colliders.len() {
                if self.sub_colliders[i].bounding_collides_with(&rhs.sub_colliders[j]) && self.sub_colliders[i].internals_collide_with(&rhs.sub_colliders[j]) {
                    
                    return Collision::Yes
                }
            }
        }
        Collision::No
    }

    pub fn get_moved(&self, position:Vec3Df, orientation:Orientation) -> Self {
        let mut clone = self.clone();
        let rotation = Rotation::from_orientation(orientation);
        clone.global_aabb = clone.global_aabb.rotate(&rotation);
        clone.global_aabb += position;
        for sub in &mut clone.sub_colliders {
            *sub = sub.get_moved(position, &rotation);
        }
        clone
    }
}

#[derive(Clone, Debug, ToBytes, FromBytes, PartialEq)]
pub enum BoundingCollider {
    AABB(AABB),
    BS(BoundingSphere)
}

impl BoundingCollider {
    pub fn collides_with_world(&self, world:&GameMap<CoolVoxel, Road>) -> bool {
        match self {
            Self::AABB(aabb) => aabb.collision_world(world),
            Self::BS(sphere) => sphere.collision_world(world)
        }
    }
    pub fn point_inside(&self, point:Vec3Df) -> bool {
        match self {
            Self::AABB(aabb) => aabb.collision_point(&point),
            Self::BS(sphere) => sphere.point_inside(point)
        }
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub struct SubCollider {
    bounding_collider:BoundingCollider,
    internal_colliders:Vec<InternalCollider>
}

impl SubCollider {
    pub fn new(bounding_collider:BoundingCollider, internal_colliders:Vec<InternalCollider>) -> Self {
        Self { bounding_collider, internal_colliders }
    }
    pub fn bounding_collides_with(&self, rhs:&Self) -> bool {
        match &self.bounding_collider {
            BoundingCollider::AABB(aabb1) => match &rhs.bounding_collider {
                BoundingCollider::AABB(aabb2) => aabb1.collision_aabb(aabb2),
                BoundingCollider::BS(sphere2) => aabb_sphere_collision(aabb1, sphere2),
            },
            BoundingCollider::BS(sphere1) => match &rhs.bounding_collider {
                BoundingCollider::AABB(aabb2) => aabb_sphere_collision(aabb2, sphere1),
                BoundingCollider::BS(sphere2) => sphere1.collides_with_sphere(sphere2),
            },
        }
    }
    pub fn internals_collide_with(&self, rhs:&Self) -> bool {
        for i in 0..self.internal_colliders.len() {
            for j in 0..rhs.internal_colliders.len() {
                if self.internal_colliders[i].collides_with(&rhs.internal_colliders[j]) {
                    return true
                }
            }
        }
        false
    }
    pub fn get_moved(&self, position:Vec3Df, rotation:&Rotation) -> Self {
        let mut clone = self.clone();
        clone.bounding_collider = match clone.bounding_collider {
            BoundingCollider::AABB(aabb) => BoundingCollider::AABB(aabb.rotate(rotation) + position),
            BoundingCollider::BS(bs) => BoundingCollider::BS(BoundingSphere::new(rotation.rotate(bs.center) + position, bs.radius))
        };
        for internal in &mut clone.internal_colliders {
            *internal = match internal {
                InternalCollider::AABB(aabb) => InternalCollider::AABB(aabb.rotate(rotation) + position),
                InternalCollider::BS(bs) => InternalCollider::BS(BoundingSphere::new(rotation.rotate(bs.center) + position, bs.radius)),
                InternalCollider::Complex(face) => InternalCollider::Complex(face.rotate_around_origin(rotation) + position),
            }
        }
        clone
    }
}

#[derive(Clone, ToBytes, FromBytes, PartialEq)]
pub enum InternalCollider {
    AABB(AABB),
    BS(BoundingSphere),
    Complex(FixedConvexFace<3>)
}

impl InternalCollider {
    pub fn collides_with(&self, rhs:&Self) -> bool {
        match self {
            InternalCollider::AABB(aabb1) => match rhs {
                InternalCollider::AABB(aabb2) => aabb1.collision_aabb(aabb2),
                InternalCollider::BS(bs2) => aabb_sphere_collision(aabb1, bs2),
                InternalCollider::Complex(face2) => aabb_triangle_collision(aabb1, face2),
            },
            InternalCollider::BS(bs1) => match rhs {
                InternalCollider::AABB(aabb2) => aabb_sphere_collision(aabb2, bs1),
                InternalCollider::BS(bs2) => bs1.collides_with_sphere(bs2),
                InternalCollider::Complex(face2) => face2.intersect_with(&Sphere::new(bs1.center, bs1.radius)),
            },
            InternalCollider::Complex(face1) => match rhs {
                InternalCollider::AABB(aabb2) =>  aabb_triangle_collision(aabb2, face1),
                InternalCollider::BS(bs2) => face1.intersect_with(&Sphere::new(bs2.center, bs2.radius)),
                InternalCollider::Complex(face2) => face1.intersect_with(face2),
            }
        }
    }
}