use macroquad::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn phase_color_bright() -> Color { color_u8!(217, 232, 227, 255) }
fn phase_color_dark() -> Color { color_u8!(23, 43, 54, 255) }

const GRID_WIDTH: usize = 200;
const GRID_HEIGHT: usize = 200;
const J: f32 = 1.0;

const TEMP_MIN: f32 = 0.1;
const TEMP_MAX: f32 = 1.5;
const CHEM_MIN: f32 = -4.0;
const CHEM_MAX: f32 = 0.0;

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
                if rand::gen_range(0, 2) == 1 {
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
        for _ in 0..(self.width * self.height) {
            let x = rand::gen_range(0, self.width);
            let y = rand::gen_range(0, self.height);
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
                Site::Empty => -J * neighbor_molecules as f32,
                Site::Molecule =>  J * neighbor_molecules as f32,
            };
            let delta_n = match current_site {
                Site::Empty => 1.0,
                Site::Molecule => -1.0,
            };
            let delta_h = delta_e - chem_potential * delta_n;

            if delta_h <= 0.0 || rand::gen_range(0.0, 1.0) < (-delta_h / temp).exp() {
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

struct DensityHistory {
    values: Vec<f32>,
    max_length: usize,
}

impl DensityHistory {
    fn new(max_length: usize) -> Self {
        Self {
            values: Vec::with_capacity(max_length),
            max_length,
        }
    }

    fn push(&mut self, value: f32) {
        self.values.push(value);
        if self.values.len() > self.max_length {
            self.values.remove(0);
        }
    }
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
    if d <= 0.0 || d >= 1.0 || temp <= 0.0 { return -f32::INFINITY; }
    let energy_term = (2.0 * J * d * d + chem_potential * d) / temp;
    let entropy_term = d * d.ln() + (1.0 - d) * (1.0 - d).ln();
    energy_term - entropy_term
}

#[macroquad::main("Phase Transition Simulation - egui")]
async fn main() {
    let mut temperature: f32 = 1.2;
    let mut chemical_potential = -2.0;
    let mut lattice = Lattice::new(GRID_WIDTH, GRID_HEIGHT);
    let mut logger = SimulationLogger::new();
    let mut step_counter: u64 = 0;
    let mut density_history = DensityHistory::new(1000);

    let phase_diagram = PhaseDiagram::new(100, 100, (TEMP_MIN, TEMP_MAX), (CHEM_MIN, CHEM_MAX));

    loop {
        if is_key_pressed(KeyCode::Space) { 
            lattice = Lattice::new(GRID_WIDTH, GRID_HEIGHT); 
        }
        if is_key_pressed(KeyCode::S) {
            let filename = format!("simulation_{}_steps.csv", step_counter);
            let _ = logger.save_csv(filename);
        }

        lattice.step(temperature, chemical_potential);
        step_counter += 1;
        let density = lattice.molecule_count() as f32 / (lattice.width * lattice.height) as f32;
        logger.record(step_counter, temperature, chemical_potential, density);
        density_history.push(density);

        clear_background(Color::from_rgba(30, 30, 35, 255));

        let sw = screen_width();
        let sh = screen_height();
        let lattice_size = (sh - 40.0).min(sw * 0.55);
        let lattice_rect = Rect::new(20.0, 20.0, lattice_size, lattice_size);
        
        lattice.draw(lattice_rect);
        draw_rectangle_lines(lattice_rect.x, lattice_rect.y, lattice_rect.w, lattice_rect.h, 2.0, GRAY);

        egui_macroquad::ui(|egui_ctx| {
            draw_egui_ui(
                egui_ctx,
                &mut temperature,
                &mut chemical_potential,
                &mut lattice,
                &density_history,
                &phase_diagram,
                density,
                step_counter,
                &lattice_rect,
            );
        });

        egui_macroquad::draw();
        
        let target_fps = 30.0;
        let frame_time = 1.0 / target_fps;
        let elapsed = get_frame_time();
        if elapsed < frame_time {
            let sleep_time = frame_time - elapsed;
            std::thread::sleep(std::time::Duration::from_secs_f32(sleep_time));
        }
        
        next_frame().await
    }
}

fn draw_egui_ui(
    ctx: &egui_macroquad::egui::Context,
    temperature: &mut f32,
    chemical_potential: &mut f32,
    lattice: &mut Lattice,
    density_history: &DensityHistory,
    phase_diagram: &PhaseDiagram,
    density: f32,
    step_counter: u64,
    lattice_rect: &Rect,
) {
    use egui_macroquad::egui;
    
    let panel_x = lattice_rect.x + lattice_rect.w + 20.0;
    let below_lattice_y = lattice_rect.y + lattice_rect.h + 10.0;
    let controls_width = 320.0;
    let controls_height = 260.0;
    let density_height = 280.0;
    
    egui::Window::new("Controls")
        .default_pos([panel_x, 20.0])
        .default_size([controls_width, controls_height])
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Simulation Parameters");
            ui.separator();
            
            ui.label("Temperature (T):");
            ui.add(egui::Slider::new(temperature, TEMP_MIN..=TEMP_MAX)
                .text("T"));
            
            ui.add_space(5.0);
            ui.label("Chemical Potential (μ):");
            ui.add(egui::Slider::new(chemical_potential, CHEM_MIN..=CHEM_MAX)
                .text("μ"));
            
            ui.separator();
            
            ui.horizontal(|ui| {
                ui.label("Density:");
                ui.colored_label(egui::Color32::from_rgb(100, 200, 100), 
                    format!("{:.4}", density));
            });
            
            ui.horizontal(|ui| {
                ui.label("Step:");
                ui.colored_label(egui::Color32::from_rgb(200, 200, 100), 
                    format!("{}", step_counter));
            });
            
            ui.horizontal(|ui| {
                ui.label("FPS:");
                ui.colored_label(egui::Color32::from_rgb(100, 150, 255), 
                    format!("{}", macroquad::time::get_fps()));
            });
            
            ui.separator();
            
            if ui.button("🔄 Randomize Grid (Space)").clicked() {
                *lattice = Lattice::new(GRID_WIDTH, GRID_HEIGHT);
            }
            
            ui.label("Press S to save CSV");
        });

    egui::Window::new("Density vs Time")
        .default_pos([panel_x, 20.0 + controls_height + 10.0])
        .default_size([controls_width, density_height])
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(format!("Current: {:.4}", density));
            ui.label(format!("Samples: {}", density_history.values.len()));
            ui.separator();
            
            let plot_height = 150.0;
            let (response, painter) = ui.allocate_painter(
                egui::vec2(ui.available_width(), plot_height),
                egui::Sense::hover()
            );
            
            let rect = response.rect;
            
            if density_history.values.len() > 1 {
                let min_val = density_history.values.iter().cloned().fold(f32::INFINITY, f32::min);
                let max_val = density_history.values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                let range = (max_val - min_val).max(0.01);
                
                let points: Vec<egui::Pos2> = density_history.values
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| {
                        let x = rect.left() + (i as f32 / (density_history.values.len() - 1) as f32) * rect.width();
                        let y = rect.bottom() - ((v - min_val) / range) * rect.height();
                        egui::pos2(x, y)
                    })
                    .collect();
                
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
                
                for i in 0..5 {
                    let y = rect.top() + (i as f32 / 4.0) * rect.height();
                    painter.line_segment(
                        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                        egui::Stroke::new(0.5, egui::Color32::from_gray(40))
                    );
                }
                
                for i in 0..points.len() - 1 {
                    painter.line_segment(
                        [points[i], points[i + 1]],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0))
                    );
                }
                
                if let Some(&last) = points.last() {
                    painter.circle_filled(last, 3.0, egui::Color32::from_rgb(255, 100, 100));
                }
            } else {
                painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Collecting data...",
                    egui::FontId::default(),
                    egui::Color32::WHITE
                );
            }
        });

    let phase_width = (lattice_rect.w / 2.0 - 10.0).max(250.0);
    let phase_height = 320.0;
    
    egui::Window::new("Phase Diagram (T vs μ)")
        .default_pos([20.0, below_lattice_y])
        .default_size([phase_width, phase_height])
        .resizable(true)
        .show(ctx, |ui| {
            ui.label("Phase space visualization");
            ui.label(format!("T: {:.2}, μ: {:.2}", *temperature, *chemical_potential));
            
            let (res_t, res_c) = phase_diagram.resolution;
            let t_frac = (*temperature - phase_diagram.temp_range.0) 
                / (phase_diagram.temp_range.1 - phase_diagram.temp_range.0);
            let c_frac = (*chemical_potential - phase_diagram.chem_potential_range.0) 
                / (phase_diagram.chem_potential_range.1 - phase_diagram.chem_potential_range.0);
            
            if t_frac >= 0.0 && t_frac <= 1.0 && c_frac >= 0.0 && c_frac <= 1.0 {
                let i = (t_frac * res_t as f32).min((res_t - 1) as f32) as usize;
                let j = (c_frac * res_c as f32).min((res_c - 1) as f32) as usize;
                ui.label(format!("Equilibrium density: {:.3}", phase_diagram.densities[i][j]));
            }
            
            ui.separator();
            
            let plot_size = egui::vec2(ui.available_width(), 250.0);
            let (response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());
            let rect = response.rect;
            
            let (res_t, res_c) = phase_diagram.resolution;
            let cell_w = rect.width() / res_t as f32;
            let cell_h = rect.height() / res_c as f32;
            
            for i in 0..res_t {
                for j in 0..res_c {
                    let density = phase_diagram.densities[i][j].clamp(0.0, 1.0);
                    
                    let bright = phase_color_bright();
                    let dark = phase_color_dark();
                    let r = ((bright.r + density * (dark.r - bright.r)) * 255.0) as u8;
                    let g = ((bright.g + density * (dark.g - bright.g)) * 255.0) as u8;
                    let b = ((bright.b + density * (dark.b - bright.b)) * 255.0) as u8;
                    let color = egui::Color32::from_rgb(r, g, b);
                    
                    let x = rect.left() + i as f32 * cell_w;
                    let y = rect.top() + j as f32 * cell_h;
                    let cell_rect = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(cell_w, cell_h)
                    );
                    painter.rect_filled(cell_rect, 0.0, color);
                }
            }
            
            if t_frac >= 0.0 && t_frac <= 1.0 && c_frac >= 0.0 && c_frac <= 1.0 {
                let marker_x = rect.left() + t_frac * rect.width();
                let marker_y = rect.top() + c_frac * rect.height();
                painter.circle_filled(
                    egui::pos2(marker_x, marker_y),
                    5.0,
                    egui::Color32::from_rgb(255, 0, 0)
                );
            }
        });

    let energy_x = 20.0 + phase_width + 10.0;
    let energy_width = (lattice_rect.w - phase_width - 10.0).max(250.0);
    
    egui::Window::new("Free Energy Density f(ρ)")
        .default_pos([energy_x, below_lattice_y])
        .default_size([energy_width, phase_height])
        .resizable(true)
        .show(ctx, |ui| {
            ui.label(format!("T: {:.2}, μ: {:.2}", *temperature, *chemical_potential));
            ui.label(format!("Current ρ: {:.4}", density));
            ui.separator();
            
            let steps = 100;
            let mut points = Vec::new();
            let mut max_f = -f32::INFINITY;
            let mut min_f = f32::INFINITY;
            
            for i in 0..=steps {
                let d = i as f32 / steps as f32;
                let f = calculate_ftc(d, *temperature, *chemical_potential);
                if f.is_finite() {
                    points.push((d, f));
                    if f > max_f { max_f = f; }
                    if f < min_f { min_f = f; }
                }
            }
            
            let plot_size = egui::vec2(ui.available_width(), 250.0);
            let (response, painter) = ui.allocate_painter(plot_size, egui::Sense::hover());
            let rect = response.rect;
            
            painter.rect_filled(rect, 0.0, egui::Color32::from_gray(20));
            
            for i in 0..5 {
                let y = rect.top() + (i as f32 / 4.0) * rect.height();
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    egui::Stroke::new(0.5, egui::Color32::from_gray(40))
                );
            }
            
            if !points.is_empty() && max_f > min_f {
                let range = max_f - min_f;
                let plot_points: Vec<egui::Pos2> = points
                    .iter()
                    .map(|&(d, f)| {
                        let x = rect.left() + d * rect.width();
                        let y = rect.bottom() - ((f - min_f) / range) * rect.height();
                        egui::pos2(x, y)
                    })
                    .collect();
                
                for i in 0..plot_points.len() - 1 {
                    painter.line_segment(
                        [plot_points[i], plot_points[i + 1]],
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 0))
                    );
                }
                
                let f_current = calculate_ftc(density, *temperature, *chemical_potential);
                if f_current.is_finite() {
                    let marker_x = rect.left() + density * rect.width();
                    let marker_y = rect.bottom() - ((f_current - min_f) / range) * rect.height();
                    painter.circle_filled(
                        egui::pos2(marker_x, marker_y),
                        4.0,
                        egui::Color32::from_rgb(255, 0, 0)
                    );
                }
            }
        });
}

