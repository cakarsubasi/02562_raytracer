use std::{time::Duration, ops::{AddAssign, DivAssign}};

#[derive(Debug, Copy, Clone, Default)]
pub struct BvhConstructionTime {
    pub morton_codes: Duration,
    pub radix_sort: Duration,
    pub treelet_init: Duration,
    pub treelet_build: Duration,
    pub upper_tree: Duration,
    pub flattening: Duration,
}

impl BvhConstructionTime {
    pub fn total(&self) -> Duration {
        self.morton_codes + self.radix_sort + self.treelet_init + self.treelet_build + self.upper_tree + self.flattening
    }

    pub fn display(&self, text: &str) -> Self {
        println!("{}", text);
        println!("  morton_codes:  {:?}", self.morton_codes);
        println!("  radix_sort:    {:?}", self.radix_sort);
        println!("  treelet_init:  {:?}", self.treelet_init);
        println!("  treelet_build: {:?}", self.treelet_build);
        println!("  upper_tree:    {:?}", self.upper_tree);
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

impl AddAssign<BvhConstructionTime> for BvhConstructionTime {
    fn add_assign(&mut self, rhs: Self) {
        self.morton_codes += rhs.morton_codes;
        self.radix_sort += rhs.radix_sort;
        self.treelet_init += rhs.treelet_init;
        self.treelet_build += rhs.treelet_build;
        self.upper_tree += rhs.upper_tree;
        self.flattening += rhs.flattening;
    }
}

impl DivAssign<u32> for BvhConstructionTime {
    fn div_assign(&mut self, rhs: u32) {
        self.morton_codes /= rhs;
        self.radix_sort /= rhs;
        self.treelet_init /= rhs;
        self.treelet_build /= rhs;
        self.upper_tree /= rhs;
        self.flattening /= rhs;
    }
}