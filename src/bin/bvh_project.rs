use raytracer_wgpu_lib::data_structures::bvh_util::BvhConstructionTime;
use raytracer_wgpu_lib::data_structures::hlbvh::Bvh;
use raytracer_wgpu_lib::data_structures::bsp_tree::BspTree;
use raytracer_wgpu_lib::mesh::Mesh;

use std::ops::{AddAssign, DivAssign};
use std::time::{Instant, Duration};

/// Benchmark binary for the BVH project

fn main() {

    // teapot: 6,320 bunny: 69,451 dragon: 871,414
    let model_teapot = Mesh::from_obj("res/models/teapot.obj").expect("Failed to load model");
    let model_bunny = Mesh::from_obj("res/models/bunny.obj").expect("Failed to load model");
    let model_dragon = Mesh::from_obj("res/models/dragon.obj").expect("Failed to load model");
    let runs = 100;

    println!("Benchmarking with {runs} samples.\n");

    // Performance scaling with triangles
    println!("Performance scaling with triangles (1/4):");
    let bvh_teapot_4_mt =
    run_bvh(&model_teapot, 4, false, runs).display("BVH: Teapot (6,320), 4, MT");
    let bvh_bunny_4_mt =
    run_bvh(&model_bunny , 4, false, runs).display("BVH: Bunny (69,451), 4, MT");
    let bvh_dragon_4_mt = 
    run_bvh(&model_dragon, 4, false, runs).display("BVH: Dragon (871,414), 4, MT");
    println!("----------------------------------");

    // Performance scaling with leaf primitives:
    println!("\nPerformance scaling with triangles (2/4):");
    run_bvh(&model_dragon, 1, false, runs).display("BVH: Dragon, 1, MT");
    run_bvh(&model_dragon, 2, false, runs).display("BVH: Dragon, 2, MT");
    bvh_dragon_4_mt.display("Dragon, 4, MT");
    run_bvh(&model_dragon, 6, false, runs).display("BVH: Dragon, 6, MT");
    let bvh_dragon_8_mt =
    run_bvh(&model_dragon, 8, false, runs).display("BVH: Dragon, 8, MT");
    run_bvh(&model_dragon, 16, false, runs).display("BVH: Dragon, 16, MT");
    println!("----------------------------------");

    // Multithreaded performance scaling:
    println!("\nMultithreaded performance scaling (3/4):");
    bvh_dragon_4_mt.display("Dragon, 4, MT");
    let bvh_dragon_4_st = 
    run_bvh(&model_dragon, 4, true, runs).display("BVH: Dragon, 4, ST");
    bvh_dragon_8_mt.display("Dragon, 4, MT");
    let bvh_dragon_8_st = 
    run_bvh(&model_dragon, 8, true, runs).display("BVH: Dragon, 8, ST");
    println!("----------------------------------");

    // Comparison with BSP
    println!("\nPerformance comparison with the BSP (4/4):");
    println!("\nTeapot:");
    bvh_teapot_4_mt.display("BVH: Teapot, 4, MT");
    run_single_bsp(&model_teapot, 20, 4, runs).display("BSP: Teapot, 4, dep: 20");
    println!("\nBunny:");
    bvh_bunny_4_mt.display("BVH: Bunny, 4, MT");
    run_single_bsp(&model_bunny , 20, 4, runs).display("BSP: Bunny , 4, dep: 20");
    println!("\nDragon, 4 leaf primitives:");
    bvh_dragon_4_st.display("Dragon, 4, ST");
    bvh_dragon_4_mt.display("Dragon, 4, MT");
    run_single_bsp(&model_dragon, 20, 4, runs).display("BSP: Dragon, 4, dep: 20");
    println!("\nDragon, 8 leaf primitives:");
    bvh_dragon_8_st.display("Dragon, 8, ST");
    bvh_dragon_8_mt.display("Dragon, 8, MT");
    run_single_bsp(&model_dragon, 20, 8, runs).display("BSP: Dragon, 8, dep: 20");
    println!("----------------------------------");

    println!("\nAll done.");
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


/// Wrapper type for benchmarking the Bsp
/// We don't really care about the Bsp performance but it is a nice comparison point

#[derive(Debug, Copy, Clone, Default)]
pub struct BspConstructionTime {
    pub subdivision: Duration,
    pub flattening: Duration,
}

impl BspConstructionTime {
    pub fn total(&self) -> Duration {
        self.subdivision + self.flattening
    }

    pub fn display(&self, text: &str) -> Self {
        println!("{}", text);
        println!("  subdivision:   {:?}", self.subdivision);
        println!("  flattening:    {:?}", self.flattening);
        println!("  total:         {:?}", self.total());
        *self
    }

    pub fn display_short(&self, text: &str) -> Self {
        println!("{}", text);
        println!("  total:         {:?}", self.total());
        *self
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