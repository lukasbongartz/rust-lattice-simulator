use macroquad::prelude::*;
mod density_plot;
use density_plot::DensityPopup;
use ::rand::{rng, Rng, random};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn phase_color_bright() -> Color { color_u8!(217, 232, 227, 255) }
fn phase_color_dark() -> Color { color_u8!(23, 43, 54, 255) }

const GRID_WIDTH: usize = 200;
const GRID_HEIGHT: usize = 200;

const J_MF: f32 = 1.0;
const Z: f32 = 4.0;
const J0: f32 = 2.0 * J_MF / Z; 

#[derive(Clone, Copy, PartialEq)]
enum Site {
    Molecule,
    Empty,
}

struct Lattice {
    grid: Vec<Vec<Site>>,
    width: usize,
    height: usize,
}

impl Lattice {
    fn new(width: usize, height: usize) -> Self {
        let mut grid = vec![vec![Site::Empty; height]; width];
        for x in 0..width {
            for y in 0..height {
                if random::<bool>() {
                    grid[x][y] = Site::Molecule;
                }
            }
        }
        Lattice { grid, width, height }
    }

    fn molecule_count(&self) -> usize {
        self.grid.iter().flatten().filter(|&&s| s == Site::Molecule).count()
    }
    
    fn step(&mut self, temp: f32, chem_potential: f32) {
        if temp <= 0.0 { return; }
        let mut rng = rng();
        for _ in 0..(self.width * self.height) {
            let x = rng.random_range(0..self.width);
            let y = rng.random_range(0..self.height);
            let current_site = self.grid[x][y];
            
            let mut neighbor_molecules = 0;
            let up = (y + self.height - 1) % self.height;
            let down = (y + 1) % self.height;
            let left = (x + self.width - 1) % self.width;
            let right = (x + 1) % self.width;
            
            if self.grid[x][up] == Site::Molecule { neighbor_molecules += 1; }
            if self.grid[x][down] == Site::Molecule { neighbor_molecules += 1; }
            if self.grid[left][y] == Site::Molecule { neighbor_molecules += 1; }
            if self.grid[right][y] == Site::Molecule { neighbor_molecules += 1; }
            
            let delta_e = match current_site {
                Site::Empty => -J0 * neighbor_molecules as f32,
                Site::Molecule =>  J0 * neighbor_molecules as f32,
            };
            let delta_n = match current_site {
                Site::Empty => 1.0,
                Site::Molecule => -1.0,
            };
            let delta_h = delta_e - chem_potential * delta_n;

            if delta_h <= 0.0 || random::<f32>() < (-delta_h / temp).exp() {
                self.grid[x][y] = match current_site {
                    Site::Molecule => Site::Empty,
                    Site::Empty => Site::Molecule,
                };
            }
        }
    }

    fn draw(&self, rect: Rect) {
        let cell_w = rect.w / self.width as f32;
        let cell_h = rect.h / self.height as f32;
        for x in 0..self.width {
            for y in 0..self.height {
                let color = match self.grid[x][y] {
                    Site::Molecule => phase_color_dark(),
                    Site::Empty => phase_color_bright(),
                };
                draw_rectangle(rect.x + x as f32 * cell_w, rect.y + y as f32 * cell_h, cell_w, cell_h, color);
            }
        }
    }
}

#[derive(PartialEq)]
enum Mode {
    UI,
    PhaseDiagram,
    FreeEnergyPlot,
}

struct PhaseDiagram {
    densities: Vec<Vec<f32>>,
    temp_range: (f32, f32),
    chem_potential_range: (f32, f32),
    resolution: (usize, usize),
}

impl PhaseDiagram {
    fn new(resolution_t: usize, resolution_c: usize, temp_range: (f32, f32), chem_potential_range: (f32, f32)) -> Self {
        let mut densities = vec![vec![0.0; resolution_c]; resolution_t];
        for i in 0..resolution_t {
            let temp = temp_range.0 + (i as f32 / resolution_t as f32) * (temp_range.1 - temp_range.0);
            for j in 0..resolution_c {
                let chem_potential = chem_potential_range.0 + (j as f32 / resolution_c as f32) * (chem_potential_range.1 - chem_potential_range.0);
                
                let mut max_f = -f32::INFINITY;
                let mut best_d = 0.0;
                for k in 1..1000 {
                    let d = k as f32 / 1000.0;
                    let f = calculate_ftc(d, temp, chem_potential);
                    if f > max_f {
                        max_f = f;
                        best_d = d;
                    }
                }
                densities[i][j] = best_d;
            }
        }
        PhaseDiagram { densities, temp_range, chem_potential_range, resolution: (resolution_t, resolution_c) }
    }

    fn draw(&self, rect: Rect, current_temp: f32, current_chem_potential: f32) {
        let (res_t, res_c) = self.resolution;
        let cell_w = rect.w / res_t as f32;
        let cell_h = rect.h / res_c as f32;

        for i in 0..res_t {
            for j in 0..res_c {
                let density = self.densities[i][j].clamp(0.0, 1.0);
                let bright = phase_color_bright();
                let dark = phase_color_dark();
                let r = bright.r + density * (dark.r - bright.r);
                let g = bright.g + density * (dark.g - bright.g);
                let b = bright.b + density * (dark.b - bright.b);
                let color = Color { r, g, b, a: 1.0 };
                draw_rectangle(rect.x + i as f32 * cell_w, rect.y + j as f32 * cell_h, cell_w, cell_h, color);
            }
        }
        
        let t_frac = (current_temp - self.temp_range.0) / (self.temp_range.1 - self.temp_range.0);
        let c_frac = (current_chem_potential - self.chem_potential_range.0) / (self.chem_potential_range.1 - self.chem_potential_range.0);
        if t_frac >= 0.0 && t_frac <= 1.0 && c_frac >= 0.0 && c_frac <= 1.0 {
            let marker_x = rect.x + t_frac * rect.w;
            let marker_y = rect.y + c_frac * rect.h;
            draw_circle(marker_x, marker_y, 5.0, RED);
        }

        draw_text("T", rect.x + rect.w / 2.0 - 5.0, rect.y + rect.h + 20.0, 20.0, WHITE);
        draw_text("µ", rect.x - 25.0, rect.y + rect.h / 2.0 - 5.0, 20.0, WHITE);
    }
}

struct SimulationLogger {
    records: Vec<(u64, f32, f32, f32)>,
}

impl SimulationLogger {
    fn new() -> Self { Self { records: Vec::new() } }

    fn record(&mut self, step: u64, temperature: f32, chem_potential: f32, density: f32) {
        self.records.push((step, temperature, chem_potential, density));
    }

    fn save_csv(&self, path: impl Into<PathBuf>) -> std::io::Result<()> {
        let path: PathBuf = path.into();
        let mut file = File::create(path)?;
        writeln!(file, "step,temperature,chem_potential,density")?;
        for (step, t, c, d) in &self.records {
            writeln!(file, "{},{},{},{}", step, t, c, d)?;
        }
        Ok(())
    }
}

fn calculate_ftc(d: f32, temp: f32, chem_potential: f32) -> f32 {
    if d <= 0.0 || d >= 1.0 || temp <= 0.0 {
        return -f32::INFINITY;
    }
    let energy_term = (J_MF * d * d + chem_potential * d) / temp;
    let entropy_term = d * d.ln() + (1.0 - d) * (1.0 - d).ln();
    energy_term - entropy_term
}

fn draw_ftc_plot(rect: Rect, temp: f32, chem_potential: f32, current_density: f32) {
    let steps = 200;
    let mut points = Vec::new();
    let mut max_f = -f32::INFINITY;
    let mut min_f = f32::INFINITY;

    for i in 0..=steps {
        let d = i as f32 / steps as f32;
        let f = calculate_ftc(d, temp, chem_potential);
        if f.is_finite() {
            points.push((d, f));
            if f > max_f { max_f = f; }
            if f < min_f { min_f = f; }
        }
    }
    
    draw_line(rect.x, rect.y + rect.h, rect.x + rect.w, rect.y + rect.h, 2.0, WHITE); // d-axis
    draw_line(rect.x, rect.y, rect.x, rect.y + rect.h, 2.0, WHITE); // f(d)-axis
    draw_text("d", rect.x + rect.w / 2.0 - 5.0, rect.y + rect.h + 20.0, 20.0, WHITE);
    draw_text("f(d)", rect.x - 45.0, rect.y + rect.h / 2.0 - 5.0, 20.0, WHITE);

    for i in 0..points.len()-1 {
        let (d1, f1) = points[i];
        let (d2, f2) = points[i+1];
        let x1 = rect.x + d1 * rect.w;
        let y1 = rect.y + rect.h - ((f1 - min_f) / (max_f - min_f).max(0.001)) * rect.h;
        let x2 = rect.x + d2 * rect.w;
        let y2 = rect.y + rect.h - ((f2 - min_f) / (max_f - min_f).max(0.001)) * rect.h;
        draw_line(x1, y1, x2, y2, 2.0, YELLOW);
    }
    
    let marker_x = rect.x + current_density * rect.w;
    let f_current = calculate_ftc(current_density, temp, chem_potential);
    if f_current.is_finite() {
        let marker_y = rect.y + rect.h - ((f_current - min_f) / (max_f - min_f).max(0.001)) * rect.h;
        draw_circle(marker_x, marker_y, 5.0, RED);
    }
}

#[macroquad::main("Phase Transition Simulation")]
async fn main() {

    let mut temperature: f32 = 0.7;
    let mut chemical_potential: f32 = -1.0;
    let mut lattice = Lattice::new(GRID_WIDTH, GRID_HEIGHT);
    let mut mode = Mode::UI;
    let mut logger = SimulationLogger::new();
    let mut step_counter: u64 = 0;
    let mut density_popup = DensityPopup::new(1000);

    let phase_diagram = PhaseDiagram::new(100, 100, (0.01, 1.0), (-2.0, 0.0));
   
    loop {
        if is_key_down(KeyCode::Up) { temperature += 0.01; }
        if is_key_down(KeyCode::Down) { temperature = (temperature - 0.01).max(0.01); }
        if is_key_down(KeyCode::Right) { chemical_potential += 0.02; }
        if is_key_down(KeyCode::Left) { chemical_potential -= 0.02; }
        if is_key_pressed(KeyCode::Space) { lattice = Lattice::new(GRID_WIDTH, GRID_HEIGHT); }
        if is_key_pressed(KeyCode::M) {
            mode = match mode {
                Mode::UI => Mode::PhaseDiagram,
                Mode::PhaseDiagram => Mode::FreeEnergyPlot,
                Mode::FreeEnergyPlot => Mode::UI,
            }
        }
        if is_key_pressed(KeyCode::D) { density_popup.toggle(); }

        lattice.step(temperature, chemical_potential);
        step_counter += 1;
        let density = lattice.molecule_count() as f32 / (lattice.width * lattice.height) as f32;
        logger.record(step_counter, temperature, chemical_potential, density);
        density_popup.record_density(density);

        clear_background(BLACK);

        let sw = screen_width();
        let sh = screen_height();
        let margin = 20.0;
        let main_panel_width = (sw / 2.0).max(10.0);

        let sim_rect = Rect::new(margin, margin, main_panel_width - margin * 2.5, sh - margin * 2.0);
        let panel_rect = Rect::new(main_panel_width + margin * 1.5, margin, main_panel_width - margin * 2.5, sh - margin * 2.0);

        lattice.draw(sim_rect);
        draw_rectangle_lines(sim_rect.x, sim_rect.y, sim_rect.w, sim_rect.h, 2.0, GRAY);

        if is_key_pressed(KeyCode::S) {
            let filename = format!("simulation_{}_steps.csv", step_counter);
            let _ = logger.save_csv(filename);
        }

        match mode {
            Mode::UI => draw_ui_panel(panel_rect, &lattice, temperature, chemical_potential),
            Mode::PhaseDiagram => phase_diagram.draw(panel_rect, temperature, chemical_potential),
            Mode::FreeEnergyPlot => {
                draw_ftc_plot(panel_rect, temperature, chemical_potential, density);
            }
        }
        let desired_w = sw * 0.40;
        let desired_h = sh * 0.28;
        let popup_w = desired_w.max(260.0).min(panel_rect.w - 40.0).min(sw - 2.0 * margin);
        let popup_h = desired_h.max(120.0).min(panel_rect.h - 40.0).min(sh - 2.0 * margin);
        let popup_x = sw - margin - popup_w;
        let popup_y = sh - margin - popup_h;
        density_popup.draw(Rect::new(popup_x, popup_y, popup_w, popup_h));
        
        next_frame().await
    }
}

/// Draws the main user interface panel with stats and controls.
fn draw_ui_panel(rect: Rect, lattice: &Lattice, temp: f32, chem_potential: f32) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, color_u8!(10, 10, 10, 200));
    
    let mut y_cursor = rect.y + 24.0;
    let heading = TextParams { font_size: 28, color: YELLOW, ..Default::default() };
    draw_text_ex("Simulation Controls", rect.x + 14.0, y_cursor, heading);
    y_cursor += 40.0;

    let _label = TextParams { font_size: 20, color: WHITE, ..Default::default() };
    
    let density = lattice.molecule_count() as f32 / (lattice.width * lattice.height) as f32;

    // Aligned label/value rows
    fn row(label: &str, value: &str, x: f32, y: f32) {
        let label_params = TextParams { font_size: 20, color: WHITE, ..Default::default() };
        let value_params = TextParams { font_size: 20, color: GREEN, ..Default::default() };
        draw_text_ex(label, x, y, label_params);
        draw_text_ex(value, x + 220.0, y, value_params);
    }
    row("FPS:", &format!("{}", get_fps()), rect.x + 14.0, y_cursor);
    y_cursor += 28.0;
    row("Temperature:", &format!("{:.2}", temp), rect.x + 14.0, y_cursor);
    y_cursor += 28.0;
    row("Chem Potential:", &format!("{:.2}", chem_potential), rect.x + 14.0, y_cursor);
    y_cursor += 28.0;
    row("Density:", &format!("{:.3}", density), rect.x + 14.0, y_cursor);
    y_cursor += 36.0;
    draw_line(rect.x + 12.0, y_cursor, rect.x + rect.w - 12.0, y_cursor, 1.0, GRAY);
    y_cursor += 20.0;

    let controls_heading = TextParams { font_size: 22, color: ORANGE, ..Default::default() };
    draw_text_ex("Controls", rect.x + 14.0, y_cursor, controls_heading);
    y_cursor += 30.0;

    let controls = TextParams { font_size: 18, color: LIGHTGRAY, ..Default::default() };
    draw_text_ex("[Up/Down] Temperature", rect.x + 14.0, y_cursor, controls.clone());
    y_cursor += 25.0;
    draw_text_ex("[Left/Right] Chem. Potential", rect.x + 14.0, y_cursor, controls.clone());
    y_cursor += 25.0;
    draw_text_ex("[Space] Randomize Grid", rect.x + 14.0, y_cursor, controls.clone());
    y_cursor += 25.0;
    draw_text_ex("[M] Change Mode", rect.x + 14.0, y_cursor, controls.clone());
    y_cursor += 25.0;
    draw_text_ex("[S] Save CSV Snapshot", rect.x + 14.0, y_cursor, controls.clone());
    y_cursor += 25.0;
    draw_text_ex("[D] Toggle Density Popup", rect.x + 14.0, y_cursor, controls.clone());
}

