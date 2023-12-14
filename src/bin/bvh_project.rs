use raytracer_wgpu_lib::data_structures::bvh_util::BvhConstructionTime;
use raytracer_wgpu_lib::data_structures::hlbvh::Bvh;
use raytracer_wgpu_lib::data_structures::bsp_tree::BspTree;
use raytracer_wgpu_lib::mesh::Mesh;

use std::ops::{AddAssign, DivAssign};
use std::time::{Instant, Duration};

fn main() {

    // teapot: 6,320 bunny: 69,451 dragon: 871,414
    let model_teapot = Mesh::from_obj("res/models/teapot.obj").expect("Failed to load model");
    let model_bunny = Mesh::from_obj("res/models/bunny.obj").expect("Failed to load model");
    let model_dragon = Mesh::from_obj("res/models/dragon.obj").expect("Failed to load model");

    run_bvh(&model_teapot, 4, false, 100).display("Teapot, 4, MT");
    run_bvh(&model_bunny , 4, false, 100).display("Bunny , 4, MT");
    run_bvh(&model_dragon, 4, false, 20).display("Dragon, 4, MT");
    run_bvh(&model_dragon, 4, true, 20).display("Dragon, 4, ST");

    //run_bvh(&model_dragon, 6, false, 20).display("Dragon, 6, MT");
    //run_bvh(&model_dragon, 8, false, 20).display("Dragon, 8, MT");
    //run_bvh(&model_dragon, 10, false, 20).display("Dragon, 10, MT");
    run_bvh(&model_dragon, 16, false, 20).display("Dragon, 16, MT");

    run_single_bsp(&model_teapot, 20, 4, 100).display("Teapot, 4, 20, bsp");
    run_single_bsp(&model_bunny , 20, 4, 100).display("Bunny , 4, 20, bsp");
    run_single_bsp(&model_dragon, 20, 4, 20).display("Dragon, 4, 20, bsp");

}

fn run_bvh(model: &Mesh, max_prims: u32, single_threaded: bool, runs: u32) -> BvhConstructionTime {
    let mut total = BvhConstructionTime::default();
    for _ in 0..runs {
        let bvh = Bvh::new(&model, max_prims, single_threaded);
        let timer = Instant::now();
        let _ = bvh.flatten();
        let _ = bvh.triangles();
        let flattening_time = timer.elapsed();
        let mut result = bvh.time;
        result.flattening = flattening_time;
        total += result;
    }
    total /= runs;
    total
}

fn run_single_bsp(model: &Mesh, max_depth: u32, max_leaf_objects: u32, runs: u32) -> BspConstructionTime {
    let mut total = BspConstructionTime::default();
    for _ in 0..runs {
        let mut current = BspConstructionTime::default();
        let mut now = Instant::now();
        let bsp = BspTree::new(model.bboxes(), max_depth, max_leaf_objects);
        current.subdivision = now.elapsed();
        now = Instant::now();
        let _ = bsp.bsp_array();
        let _ = bsp.primitive_ids();
        current.flattening = now.elapsed();
        total += current;
    }
    total /= runs;
    total
}  

#[derive(Debug, Copy, Clone, Default)]
pub struct BspConstructionTime {
    pub subdivision: Duration,
    pub flattening: Duration,
}

impl BspConstructionTime {
    pub fn total(&self) -> Duration {
        self.subdivision + self.flattening
    }

    pub fn display(&self, text: &str) {
        println!("{}", text);
        println!("subdivision: {:?}", self.subdivision);
        println!("flattening:   {:?}", self.flattening);
        println!("total:        {:?}", self.total())
    }
}

impl AddAssign<BspConstructionTime> for BspConstructionTime {
    fn add_assign(&mut self, rhs: Self) {
        self.subdivision += rhs.subdivision;
        self.flattening += rhs.flattening;
    }
}

impl DivAssign<u32> for BspConstructionTime {
    fn div_assign(&mut self, rhs: u32) {
        self.subdivision /= rhs;
        self.flattening /= rhs;
    }
}