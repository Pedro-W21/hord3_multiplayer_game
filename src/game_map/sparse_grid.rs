use std::{collections::{hash_set::Iter, HashMap, HashSet}, sync::atomic::{AtomicBool, Ordering}};

use hord3::horde::geometry::vec3d::{Vec3D, Vec3Df};
use to_from_bytes::{FromBytes, ToBytes};
use to_from_bytes_derive::{ToBytes, FromBytes};

use crate::driver::colliders::AABB;

use super::{get_float_pos, WorldChunkPos, WorldVoxelPos};



#[derive(Clone, ToBytes, FromBytes)]
pub struct SparseGrid<T: FromBytes + ToBytes> {
    grid:HashMap<WorldChunkPos,Option<usize>>,
    data:Vec<T>,
    inv_scale:f32,
    length:i32,
    width:i32,
    height:i32,
    prism_start:Vec3D<f32>,
    prism_end:Vec3D<f32>,
    f_l:f32,
    f_w:f32,
    f_h:f32,
}

impl<T: FromBytes + ToBytes> SparseGrid<T> {
    pub fn new(slot_scale:f32, start:WorldVoxelPos, end:WorldVoxelPos) -> Self {
        let world_length = (end.x - start.x);
        let world_width = (end.y - start.y);
        let world_height = (end.z - start.z);
        let length = (world_length / (slot_scale as i32)) + 1;
        let width = (world_width / (slot_scale as i32)) + 1;
        let height = (world_height / (slot_scale as i32)) + 1;
        /*let mut has_data_grid = Vec::with_capacity(length * width * height);
        for i in 0..has_data_grid.capacity() {
            has_data_grid.push(AtomicBool::new(false));
        }*/
        let inv_scale = 1.0/slot_scale;
        let prism_start = get_float_pos(start).mul_floor(inv_scale);
        let prism_end = get_float_pos(end).mul_floor(inv_scale);
        let mut grid = HashMap::with_capacity((length * width * height) as usize);
        for x in start.x.div_floor(slot_scale as i32)..end.x.div_floor(slot_scale as i32) {
            for y in start.y.div_floor(slot_scale as i32)..end.y.div_floor(slot_scale as i32) {
                for z in start.z.div_floor(slot_scale as i32)..end.z.div_floor(slot_scale as i32) {
                    let pos = Vec3D::new(x, y, z);
                    grid.insert(pos, None);
                    //dbg!(pos);
                    
                    
                }
            }
        }
        Self {prism_end, prism_start, grid, data: Vec::with_capacity((length * width * height) as usize), inv_scale, length, width, height, f_h:height as f32, f_l:length as f32, f_w:width as f32 }
    }
    pub fn add_data_to_slot(&mut self, data:T, slot:WorldChunkPos) {
        let index = self.data.len();
        self.data.push(data);
        self.grid.insert(slot, Some(index));
    }
    /*pub fn signal_data_add(&self, slot:usize) -> bool { // true if you can send an event
        if let Ok(false) = self.has_data_grid[slot].compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst) {
            true
        }
        else {
            false
        }
    }*/
    pub fn is_in(&self, point:&Vec3Df) -> bool {
        let scaled_point = point * self.inv_scale;
        scaled_point.in_origin_prism(self.f_l, self.f_w, self.f_h)
    }
    pub fn get(&self, pos:Vec3D<i32>) -> Option<usize> {
        self.grid.get(&pos).unwrap().clone()
    }
    pub fn get_if_in(&self, point:&Vec3Df) -> Option<usize> {
        match point.mul_floor(self.inv_scale).to_i32_if_in_prism(self.prism_start, self.prism_end) {
            Some(triple_coord) => self.get(triple_coord),
            None => None
        }
    }
    pub fn get_data_if_in(&self, point:&Vec3Df) -> Option<&T> {
        match self.get_if_in(point) {
            Some(id) => Some(&self.data[id]),
            None => None
        }
    }
    pub fn get_data_u_coords(&self, point:Vec3D<i32>) -> Option<&T> {
        match self.get(point) {
            Some(id) => Some(&self.data[id]),
            None => None
        }
    }
    pub fn get_data_mut_if_in(&mut self, point:&Vec3Df) -> Option<&mut T> {
        match self.get_if_in(point) {
            Some(id) => Some(&mut self.data[id]),
            None => None
        }
    }
    pub fn get_data_mut_u_coords(&mut self, point:Vec3D<i32>) -> Option<&mut T> {
        match self.get(point) {
            Some(id) => Some(&mut self.data[id]),
            None => None
        }
    }
    pub fn vec3Df_to_i32(&self, point:&Vec3Df) -> Option<Vec3D<i32>> {
        point.mul_floor(self.inv_scale).to_i32_if_in_prism(self.prism_start, self.prism_end)
    }
    pub fn get_data_id_for_slot(&self, slot:Vec3D<i32>) -> Option<usize> {
        self.grid.get(&slot).unwrap().clone()
    }
    pub fn modify_data_at<F:Fn(&mut T)>(&mut self, data_id:usize, func:&F) {
        func(&mut self.data[data_id])
    }
}

pub type SetGrid = SparseGrid<Vec<HashSet<usize>>>;
 

impl SetGrid {
    pub fn get_iter_from_to<'a>(&'a self, start:Vec3Df, stop:Vec3Df, index_in_vec:usize, size:f32) -> SGIterator<'a> {
        
        let (mut start, mut stop) = AABB::new(start, stop).get_both_points();
        start -= Vec3Df::all_ones() * size * 2.0;
        stop += Vec3Df::all_ones() * size * 2.0 ;

        let start_usize = start.mul_floor(self.inv_scale).to_i32_prism_clamped(self.prism_start, self.prism_end);
        let stop_usize = stop.mul_floor(self.inv_scale).to_i32_prism_clamped(self.prism_start, self.prism_end);
 /* 
        start_usize.0 -= 1.clamp(0, start_usize.0);
        start_usize.1 -= 1.clamp(0, start_usize.1);
        start_usize.2 -= 1.clamp(0, start_usize.2);

        stop_usize.0 = (stop_usize.0 + 1).clamp(0, self.length - 1);
        stop_usize.1 = (stop_usize.1 + 1).clamp(0, self.width - 1);
        stop_usize.2 = (stop_usize.2 + 1).clamp(0, self.height - 1);
*/
        let x_limit = stop_usize.x - start_usize.x;
        let y_limit = stop_usize.y - start_usize.y;
        let z_limit = stop_usize.z - start_usize.z;
        dbg!(x_limit, y_limit, z_limit);

        let mut iteraor = SGIterator { grid: self, current_set: None, current_pos: Vec3D::zero(), x_limit, y_limit, z_limit, index_in_vec };
        iteraor.initial_iterator_update();
        iteraor
    }
    fn add_to_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:Vec3D<i32>, vec_index:usize, add:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        self.data[index][vec_index].insert(add);
    }
    fn remove_from_hashset<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:Vec3D<i32>, vec_index:usize, remove:usize) {
        let index = self.make_sure_data_exists_at::<VEC_LENGTH, SET_CAPACITY>(grid_slot);
        self.data[index][vec_index].remove(&remove);
    }
    fn make_sure_data_exists_at<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, grid_slot:Vec3D<i32>) -> usize {
        match self.grid.get(&grid_slot).unwrap() {
            Some(data_id) => *data_id,
            None => {
                let len = self.data.len();
                let mut new_vec = Vec::with_capacity(VEC_LENGTH);
                for i in 0..VEC_LENGTH {
                    new_vec.push(HashSet::with_capacity(SET_CAPACITY));
                }
                self.data.push(new_vec);
                self.grid.insert(grid_slot, Some(len));
                len
            }
        }
    }
    pub fn apply_update<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, update:SetGridUpdate) {
        match update {
            SetGridUpdate::AddToSet { grid_slot, vec_index, add } => self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, add),
            SetGridUpdate::RemoveFromSet { grid_slot, vec_index, remove } => self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_slot, vec_index, remove),
            SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id } => {
                self.remove_from_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_start, vec_index, id);
                self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(grid_end, vec_index, id);
            }
        }
    }
    fn exists_at(&self, grid_slot:Vec3D<i32>, vec_index:usize, id:usize) -> bool {
        //dbg!(grid_slot);
        match self.grid.get(&grid_slot).unwrap() {
            Some(data) => self.data[*data][vec_index].contains(&id),
            None => false
        }
    }
    fn len_at(&self, grid_slot:Vec3D<i32>, vec_index:usize) {
        match self.grid.get(&grid_slot).unwrap() {
            Some(data) => {dbg!(self.data[*data][vec_index].len());},
            None => ()
        }
    }
    pub fn get_point_move_update(&self, start:&Vec3Df, end:&Vec3Df, id:usize, vec_index:usize) -> Option<SetGridUpdate> {
        match self.vec3Df_to_i32(start) {
            Some(grid_start) => match self.vec3Df_to_i32(end) {
                Some(grid_end) => if grid_start != grid_end {
                    Some(SetGridUpdate::MoveFromTo { grid_start, grid_end, vec_index, id })
                }
                else if !self.exists_at(grid_end, vec_index, id) {
                    Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id })
                }
                else {
                    None
                },
                None => Some(SetGridUpdate::RemoveFromSet { grid_slot: grid_start, vec_index, remove:id })
            },
            None => match self.vec3Df_to_i32(end) {
                Some(grid_end) => Some(SetGridUpdate::AddToSet { grid_slot: grid_end, vec_index, add:id }),
                None => None
            }
        }
    }
    pub fn add_to_prism_slow<const VEC_LENGTH:usize, const SET_CAPACITY:usize>(&mut self, start:&Vec3Df, stop:&Vec3Df, id:usize, vec_index:usize) {
        let (start, stop) = AABB::new(*start, *stop).get_both_points();

        let start_usize = start.mul_floor(self.inv_scale).to_i32_prism_clamped(self.prism_start, self.prism_end);
        let stop_usize = stop.mul_floor(self.inv_scale).to_i32_prism_clamped(self.prism_start, self.prism_end);
        for x in start_usize.x..=stop_usize.x {
            for y in start_usize.y..=stop_usize.y {
                for z in start_usize.z..=stop_usize.z {
                    self.add_to_hashset::<VEC_LENGTH, SET_CAPACITY>(Vec3D::new(x, y, z), vec_index, id);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum SetGridUpdate {
    AddToSet {grid_slot:Vec3D<i32>, vec_index:usize, add:usize},
    RemoveFromSet {grid_slot:Vec3D<i32>, vec_index:usize, remove:usize},
    MoveFromTo {grid_start:Vec3D<i32>, grid_end:Vec3D<i32>, vec_index:usize, id:usize}
}



pub struct SGIterator<'a> {
    grid:&'a SparseGrid<Vec<HashSet<usize>>>,
    current_set:Option<Iter<'a, usize>>,
    current_pos:Vec3D<i32>,
    x_limit:i32,
    y_limit:i32,
    z_limit:i32,
    index_in_vec:usize
}

impl<'a> SGIterator<'a> {
    fn update_pos(&mut self) -> bool {
        if self.current_pos.x == self.x_limit {
            if self.current_pos.y == self.y_limit {
                if self.current_pos.z == self.z_limit {
                    /**/
                    false
                }
                else {
                    self.current_pos.x = 0;
                    self.current_pos.y = 0;
                    self.current_pos.z += 1;
                    true
                }
            }
            else {
                self.current_pos.x = 0;
                self.current_pos.y += 1;
                true
            }
        }
        else {
            self.current_pos.x += 1;
            true
        }
    }
    fn next_slot(&mut self) -> Option<usize> {
        while self.update_pos() {
            if let Some(value) = self.update_iterator() {
                return Some(value)
            }
        }
        if !self.update_pos() {
            match &self.current_set {
                Some(set) => {
                    None
                },
                None => match self.grid.grid.get(&self.current_pos).unwrap() {
                    Some(data_id) => {
                        let mut new_iterator = self.grid.data[*data_id][self.index_in_vec].iter();
                        match new_iterator.next() {
                            Some(value) => {
                                self.current_set = Some(new_iterator);
                                Some(*value)
                            },
                            None => None
                        }
                    },
                    None => {
                        None
                    }
                }
            }
        }
        else {
            None
        }
    }
    fn update_iterator(&mut self) -> Option<usize> {
        match self.grid.grid.get(&self.current_pos).unwrap() {
            Some(data_id) => {
                let mut new_iterator = self.grid.data[*data_id][self.index_in_vec].iter();
                match new_iterator.next() {
                    Some(value) => {
                        self.current_set = Some(new_iterator);
                        Some(*value)
                    },
                    None => None
                }
            },
            None => None
        }
    }
    fn initial_iterator_update(&mut self) {
        match self.grid.grid.get(&self.current_pos).unwrap() {
            Some(data_id) => {
                let new_iterator = self.grid.data[*data_id][self.index_in_vec].iter();
                self.current_set = Some(new_iterator);
            },
            None => ()
        }
    }
} 

impl<'a> Iterator for SGIterator<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self.current_set.as_mut() {
            Some(iterator) => match iterator.next() {
                Some(data) => {
                    Some(*data)
                },
                None => self.next_slot(),
            },
            None => self.next_slot(),
        }
    }
}