use std::{collections::HashSet, f32::consts::PI, sync::Arc};

use hord3::{defaults::default_rendering::vectorinator_binned::{meshes::{Mesh, MeshLOD, MeshLODS, MeshLODType, MeshTriangles}, shapes_to_tris::{cylinder_to_render_comp, vec_to_complex, vec_to_complex_rand}, textures::rgb_to_argb}, horde::geometry::{rotation::Rotation, shapes_3d::{Cylinder, FixedRegularFace, Quad, Sphere, Square}, vec3d::{Vec3D, Vec3Df}}};

pub fn line_3D_from_to(start:Vec3Df, stop:Vec3Df, texture:u32, side:f32, field:u32, light:(u8,u8,u8)) -> MeshLOD {
    let (yaw, pitch) = start.get_orient_vers(&stop);
    let cylinder = Cylinder::<4>::new(Square::new_square(side) + start, start.dist(&stop)).rotate_around_barycenter(&Rotation::new_from_euler(PI/4.0, 0.0, 0.0)).rotate_around_base_center(&Rotation::new_from_euler(yaw, pitch, 0.0));
    cylinder_to_render_comp(&cylinder, texture, texture, texture, false, &vec![light ; 4], &vec![light ; 4], field, field, field, true, true)
}
pub fn simple_line(start:Vec3Df, stop:Vec3Df, texture:u32, light:(u8,u8,u8)) -> MeshLOD {
    let side = 0.25;
    let mut mesh = line_3D_from_to(
        start,
        stop,
        texture, side, 0, light);
    mesh
}

pub fn simple_prism(start:Vec3Df, stop:Vec3Df, texture:u32, light:(u8,u8,u8)) -> MeshLOD {
    let base = Quad::new([
        start,
        Vec3Df::new(stop.x, start.y, start.z),
        Vec3Df::new(stop.x, stop.y, start.z),
        Vec3Df::new(start.x, stop.y, start.z),
    ]);
    let height = (stop.z - start.z).abs();
    let mut mesh = vec_to_complex_rand(&Cylinder::<4>::new(FixedRegularFace::from_points(base.get_points()), height).get_triangles(false).get_all_tris_raw(), (texture, texture), 0);
    mesh
}

pub fn lit_selection_cube(start:Vec3Df, stop:Vec3Df, texture:u32, light:(u8,u8,u8)) -> MeshLOD {
    let side = 0.25;
    let mut mesh = line_3D_from_to(
        start,
        Vec3Df::new(stop.x, start.y, start.z),
        texture, side, 0, light);
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(stop.x, start.y, start.z),
        texture, side, 0, light));
    
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(start.x, stop.y, start.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(start.x, start.y, stop.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(start.x, start.y, stop.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, stop.y, stop.z),
        stop,
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, start.y, stop.z),
        stop,
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, stop.y, start.z),
        stop,
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, start.y, start.z),
        Vec3Df::new(stop.x, start.y, stop.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, stop.y, start.z),
        Vec3Df::new(start.x, stop.y, stop.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(stop.x, start.y, start.z),
        Vec3Df::new(stop.x, stop.y, start.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, stop.y, start.z),
        Vec3Df::new(stop.x, stop.y, start.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, start.y, stop.z),
        Vec3Df::new(stop.x, start.y, stop.z),
        texture, side, 0, light));
    mesh.merge_with(line_3D_from_to(
        Vec3Df::new(start.x, start.y, stop.z),
        Vec3Df::new(start.x, stop.y, stop.z),
        texture, side, 0, light));
    mesh
}

pub fn selection_cube(start:Vec3Df, stop:Vec3Df, texture:u32) -> MeshLOD {
    lit_selection_cube(start, stop, texture, (255,255,255))
}

pub fn model_to_wireframe(model:MeshLOD) -> MeshLOD {
    let mut mesh = MeshLOD::new(Vec::new(), Vec::new(), Vec::new(), MeshTriangles::with_capacity(1024));

    let mut in_set = HashSet::with_capacity(1024);


    for i in 0..model.triangles.len() {
        let tri_indices = model.triangles.get_indices_for_triangle(i);
        let tri = model.triangles.get_triangle(&model.x, &model.y, &model.z, i);
        if !in_set.contains(&(tri_indices[0], tri_indices[1])) && !in_set.contains(&(tri_indices[1], tri_indices[0])) {
            mesh.merge_with(
                line_3D_from_to(
                    Vec3Df::new(tri.p1.pos.x, tri.p1.pos.y, tri.p1.pos.z),
                    Vec3Df::new(tri.p2.pos.x, tri.p2.pos.y, tri.p2.pos.z),
                    7, 0.1, 0, (255,255,255))
            );
            in_set.insert((tri_indices[0], tri_indices[1]));
        }
        if !in_set.contains(&(tri_indices[1], tri_indices[2])) && !in_set.contains(&(tri_indices[2], tri_indices[1])) {
            mesh.merge_with(
                line_3D_from_to(
                    Vec3Df::new(tri.p2.pos.x, tri.p2.pos.y, tri.p2.pos.z),
                    Vec3Df::new(tri.p3.pos.x, tri.p3.pos.y, tri.p3.pos.z),
                    7, 0.1, 0, (255,255,255))
            );
            in_set.insert((tri_indices[1], tri_indices[2]));
        }
        if !in_set.contains(&(tri_indices[2], tri_indices[0])) && !in_set.contains(&(tri_indices[0], tri_indices[2])) {
            mesh.merge_with(
                line_3D_from_to(
                    Vec3Df::new(tri.p1.pos.x, tri.p1.pos.y, tri.p1.pos.z),
                    Vec3Df::new(tri.p3.pos.x, tri.p3.pos.y, tri.p3.pos.z),
                    7, 0.1, 0, (255,255,255))
            );
            in_set.insert((tri_indices[2], tri_indices[0]));
        } 
    }

    mesh
}

pub fn xyz_axis() -> MeshLOD {
    // Texture sets 4,5,6
    let start = Vec3Df::zero();
    let side = 0.25;
    let mut mesh = line_3D_from_to(
        start,
        Vec3Df::new(1.0,0.0,0.0),
        4, side, 0, (255,255,255));
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(0.0,1.0,0.0),
        5, side, 0, (255,255,255)));
    
    mesh.merge_with(line_3D_from_to(
        start,
        Vec3Df::new(0.0,0.0,1.0),
        6, side, 0, (255,255,255)));
    mesh
}

pub fn xyz_mesh() -> Mesh {
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(xyz_axis()))]),"XYZ_MESH".to_string(), 2.0)
}

pub fn sphere_mesh() -> Mesh {
    let sphere = Sphere::new(Vec3D::zero(), 2.0).get_triangles::<4>(false);
    let tris = vec_to_complex(&sphere, &vec![8 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(tris))]),"SPHERE_MESH".to_string(), 2.0)
}

pub fn wireframe_sphere_mesh() -> Mesh {
    let sphere = Sphere::new(Vec3D::zero(), 2.0).get_triangles::<4>(true);
    let tris = model_to_wireframe(vec_to_complex(&sphere, &vec![8 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(tris))]),"WIREFRAME_SPHERE_MESH".to_string(), 2.0)
}

pub fn textured_sphere_mesh() -> Mesh {
    let sphere = Sphere::new(Vec3D::zero(), 2.0).get_triangles::<4>(false);
    let tris = vec_to_complex(&sphere, &vec![2 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(tris))]),"TEXTURED_SPHERE_MESH".to_string(), 2.0)
}

pub fn clustered_ent_mesh() -> Mesh {
    let sphere = Sphere::new(Vec3D::zero(), 1.0).get_triangles::<4>(false);
    let mut tris = vec_to_complex(&sphere, &vec![2 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(-1.5, -1.5, 0.0), 1.0).get_triangles::<2>(false), &vec![4 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -3.0, 0.0), 1.0).get_triangles::<2>(false), &vec![5 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(1.5, -1.5, 0.0), 1.0).get_triangles::<2>(false), &vec![6 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));

    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(tris))]),"CLUSTERED_ENT_MESH".to_string(), 10.0)

}

pub fn spread_out_ent_mesh() -> Mesh {
    let sphere = Sphere::new(Vec3D::zero(), 1.0).get_triangles::<4>(false);
    let mut tris = vec_to_complex(&sphere, &vec![2 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -1.5, 0.0), 1.0).get_triangles::<2>(false), &vec![4 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -3.0, 0.0), 1.0).get_triangles::<2>(false), &vec![5 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -4.5, 0.0), 1.0).get_triangles::<2>(false), &vec![6 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(tris))]),"SPREAD_OUT_ENT_MESH".to_string(), 10.0)
}

pub fn grey_sphere_mesh() -> Mesh {
    let sphere = Sphere::new(Vec3D::zero(), 1.0).get_triangles::<4>(true);
    let mut tris = vec_to_complex(&sphere, &vec![5 ; sphere.len()], &vec![[(128,128,128) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);
    
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(tris))]),"GREY_MESH".to_string(), 2.0)
        
}

pub fn second_spread_out_ent_mesh() -> Mesh {
    let sphere = Sphere::new(Vec3D::zero(), 1.0).get_triangles::<4>(false);
    let mut tris = vec_to_complex(&sphere, &vec![2 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]);

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -1.5, 0.0), 1.0).get_triangles::<2>(false), &vec![4 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -3.0, 0.0), 1.0).get_triangles::<2>(false), &vec![5 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));

    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -4.5, 0.0), 1.0).get_triangles::<2>(false), &vec![6 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));
    
    tris.merge_with(vec_to_complex(&Sphere::new(Vec3D::new(0.0, -6.0, 0.0), 1.0).get_triangles::<2>(false), &vec![7 ; sphere.len()], &vec![[(255,255,255) ; 3] ; sphere.len()], &vec![[(0.0, 0.0), (0.0, 1.0), (1.0,1.0)] ; sphere.len()],& vec![0 ; sphere.len()]));
    Mesh::new(MeshLODS::new(vec![MeshLODType::Mesh(Arc::new(tris))]),"SECOND_SPREAD_OUT_ENT_MESH".to_string(), 10.0)
}