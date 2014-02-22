
use std::vec;

use cgmath::transform::Transform3D;
use cgmath::quaternion::Quat;
use cgmath::vector::Vec3;
use cgmath::matrix::{Mat4, ToMat4, Matrix};

pub struct Delta
{
    priv parent: uint,
    priv delta : Transform3D<f32>
}

impl Default for Delta
{
    fn default() -> Delta
    {
        Delta {
            parent: 0,
            delta: Transform3D::new(1f32, Quat::zero(), Vec3::zero()),
        }
    }
}

impl Clone for Delta
{
    fn clone(&self) -> Delta
    {
        let tras = self.delta.get();
        Delta {
            parent: self.parent.clone(),
            delta: Transform3D::new(tras.scale.clone(),
                                    tras.rot.clone(),
                                    tras.disp.clone())
        }
    }
}

pub struct Deltas
{
    priv gen: ~[(uint, uint)],
    priv delta: ~[Delta],
}

impl Clone for Deltas
{
    #[inline(never)]
    fn clone(&self) -> Deltas
    {
        let mut vec = vec::with_capacity(self.delta.len());
        unsafe {
            vec.set_len(self.delta.len());
            vec.copy_memory(self.delta.as_slice())
        }
        Deltas {
            gen: self.gen.clone(),
            delta: vec
        }
    }
}

#[deriving(Clone, Default, Eq)]
pub struct Id(uint, uint);

impl Deltas
{
    pub fn new() -> Deltas
    {
        Deltas {
            gen: ~[(0, 1)],
            delta: ~[Default::default()],
        }
    }

    pub fn root() -> Id {Id(0, 0)}

    fn get_loc(&self, id: Id) -> uint
    {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen];

        gen_offset + offset
    }

    fn add_loc(&mut self, gen: uint) -> (uint, uint)
    {
        if gen == self.gen.len() {
            let (s, len) = self.gen[gen-1];
            self.gen.push((s+len, 1));

            (s+len, 0)
        } else {
            for t in self.gen.mut_slice_from(gen+1).mut_iter() {
                let (off, len) = *t;
                *t = (off+1, len);
            }

            let (off, len) = self.gen[gen];
            self.gen[gen] = (off, len+1);

            (off+len, len)
        }
    }

    pub fn insert(&mut self, parent: Id, delta: Transform3D<f32>) -> Id
    {
        let Id(gen, pid) = parent;

        assert!(gen < self.gen.len());

        let (loc, id) = self.add_loc(gen+1);
        self.delta.insert(loc, Delta {
            parent: pid,
            delta: delta
        });

        Id(gen+1, id)
    }

    pub fn update(&mut self, id: Id, delta: Transform3D<f32>)
    {
        let loc = self.get_loc(id);
        self.delta[loc].delta = delta;
    }

    pub fn get_delta(&self, id :Id) -> Transform3D<f32>
    {
        let loc = self.get_loc(id);
        self.delta[loc].delta
    }

    pub fn get_mat(&self, id :Id) -> Mat4<f32>
    {
        let loc = self.get_loc(id);
        match id {
            Id(0, _) => {
                self.delta[loc].delta.to_mat4()
            },
            Id(gen, _) => {
                let mat = self.delta[loc].delta.to_mat4();
                let parent = Id(gen-1, self.delta[loc].parent);

                self.get_mat(parent).mul_m(&mat)
            }
        }
    }

    pub fn to_positions(&self) -> Positions
    {
        let mut mat = ~[Mat4::identity()];

        let mut last_gen_off = 0;
        for &(gen_off, len) in self.gen.slice_from(1).iter() {
            for off in range(gen_off, gen_off+len) {
                let ploc = last_gen_off + self.delta[off].parent;
                let nmat = mat[ploc].mul_m(&self.delta[off].delta.to_mat4());
                mat.push(nmat);
            }
            last_gen_off = gen_off;
        }

        Positions {
            gen: self.gen.clone(),
            pos: mat
        }
    }
}

pub struct Positions
{
    priv gen: ~[(uint, uint)],
    priv pos: ~[Mat4<f32>],
}

impl Positions
{
    pub fn root() -> Id {Id(0, 0)}

    fn get_loc(&self, id: Id) -> uint
    {
        let Id(gen, offset) = id;
        let (gen_offset, _) = self.gen[gen];

        gen_offset + offset
    }

    pub fn get_mat(&self, id :Id) -> Mat4<f32>
    {
        self.pos[self.get_loc(id)].clone()
    }
}