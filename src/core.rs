#![allow(clippy::needless_range_loop)]

use ::rand::{Rng, random, rng};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq)]
pub enum Site {
    Molecule,
    Empty,
}

pub struct Lattice {
    pub grid: Vec<Vec<Site>>,
    pub width: usize,
    pub height: usize,
    pub j: f32,
    pub epsilon0: f32,
    pub alpha: f32,
    pub num_molecules: i32,
}

impl Lattice {
    pub fn new_with_params(
        width: usize,
        height: usize,
        j: f32,
        epsilon0: f32,
        alpha: f32,
        init_density: f32,
    ) -> Self {
        let mut grid = vec![vec![Site::Empty; height]; width];
        let mut count: i32 = 0;
        for x in 0..width {
            for y in 0..height {
                if random::<f32>() < init_density {
                    grid[x][y] = Site::Molecule;
                    count += 1;
                }
            }
        }
        Lattice {
            grid,
            width,
            height,
            j,
            epsilon0,
            alpha,
            num_molecules: count,
        }
    }

    pub fn molecule_count(&self) -> usize {
        self.num_molecules as usize
    }

    pub fn step(&mut self, temp: f32, chem_potential: f32) {
        if temp <= 0.0 {
            return;
        }
        let mut rng = rng();
        let mut n_mol: i32 = self.num_molecules;
        let v_sites: f32 = (self.width * self.height) as f32;
        for _ in 0..(self.width * self.height) {
            let x = rng.random_range(0..self.width);
            let y = rng.random_range(0..self.height);
            let current_site = self.grid[x][y];

            let mut neighbor_molecules = 0;
            let up = (y + self.height - 1) % self.height;
            let down = (y + 1) % self.height;
            let left = (x + self.width - 1) % self.width;
            let right = (x + 1) % self.width;

            if self.grid[x][up] == Site::Molecule {
                neighbor_molecules += 1;
            }
            if self.grid[x][down] == Site::Molecule {
                neighbor_molecules += 1;
            }
            if self.grid[left][y] == Site::Molecule {
                neighbor_molecules += 1;
            }
            if self.grid[right][y] == Site::Molecule {
                neighbor_molecules += 1;
            }

            let j: f32 = self.j;
            let delta_e = match current_site {
                Site::Empty => -j * neighbor_molecules as f32,
                Site::Molecule => j * neighbor_molecules as f32,
            };
            let delta_n_f = match current_site {
                Site::Empty => 1.0,
                Site::Molecule => -1.0,
            };
            let delta_n_i = if matches!(current_site, Site::Empty) {
                1
            } else {
                -1
            };

            let delta_site = self.epsilon0 * delta_n_f
                - self.alpha * ((2.0 * n_mol as f32 * delta_n_f) + 1.0) / v_sites;

            let delta_h = delta_e + delta_site - chem_potential * delta_n_f;

            if delta_h <= 0.0 || random::<f32>() < (-delta_h / temp).exp() {
                self.grid[x][y] = match current_site {
                    Site::Molecule => Site::Empty,
                    Site::Empty => Site::Molecule,
                };
                n_mol += delta_n_i;
            }
        }
        self.num_molecules = n_mol;
    }
}

pub struct SimulationLogger {
    records: Vec<(u64, f32, f32, f32)>,
}

impl SimulationLogger {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn record(&mut self, step: u64, temperature: f32, chem_potential: f32, density: f32) {
        self.records
            .push((step, temperature, chem_potential, density));
    }

    pub fn save_csv(&self, path: impl Into<PathBuf>) -> std::io::Result<()> {
        let path: PathBuf = path.into();
        if let Some(dir) = path.parent() {
            if !dir.as_os_str().is_empty() {
                std::fs::create_dir_all(dir)?;
            }
        }
        let mut file = File::create(path)?;
        writeln!(file, "step,temperature,chem_potential,density")?;
        for (step, t, c, d) in &self.records {
            writeln!(file, "{step},{t},{c},{d}")?;
        }
        Ok(())
    }
}
