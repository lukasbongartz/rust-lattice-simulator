use clap::Parser;

#[path = "../core.rs"]
mod core;
use core::{Lattice, SimulationLogger};

#[derive(Parser, Debug)]
#[command(name = "ps_cli", about = "Headless lattice simulator runner")]
struct Args {
    #[arg(long, default_value_t = 150)]
    width: usize,

    #[arg(long, default_value_t = 150)]
    height: usize,

    #[arg(long, default_value_t = 10_000)]
    steps: u64,

    #[arg(long, default_value_t = 1.2)]
    temperature: f32,

    #[arg(long, default_value_t = -2.0, allow_hyphen_values = true)]
    chem_potential: f32,

    #[arg(long, default_value = "data/run.csv")]
    output: String,

    #[arg(long)]
    snapshot_csv: Option<String>,

    #[arg(long, default_value_t = 1.0)]
    interaction: f32,

    #[arg(long, default_value_t = 0.0)]
    epsilon0: f32,

    #[arg(long, default_value_t = 0.0)]
    alpha: f32,

    #[arg(long, default_value_t = 0.5)]
    init_density: f32,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut lattice = Lattice::new_with_params(
        args.width,
        args.height,
        args.interaction,
        args.epsilon0,
        args.alpha,
        args.init_density,
    );
    let mut logger = SimulationLogger::new();

    let progress_interval = (args.steps / 20).max(1);
    for step in 1..=args.steps {
        lattice.step(args.temperature, args.chem_potential);
        if step % progress_interval == 0 {
            let pct = (step as f32 / args.steps as f32) * 100.0;
            eprintln!("[ps_cli] progress: {pct:.0}%");
        }
        let density = lattice.molecule_count() as f32 / (args.width * args.height) as f32;
        logger.record(step, args.temperature, args.chem_potential, density);
    }

    logger.save_csv(args.output)?;

    if let Some(path) = args.snapshot_csv {
        save_lattice_snapshot_csv(&lattice, &path)?;
    }

    Ok(())
}

fn save_lattice_snapshot_csv(lattice: &Lattice, path: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = std::path::Path::new(path).parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }
    let mut file = std::fs::File::create(path)?;
    for y in 0..lattice.height {
        for x in 0..lattice.width {
            let v = if matches!(lattice.grid[x][y], core::Site::Molecule) {
                1
            } else {
                0
            };
            if x + 1 == lattice.width {
                writeln!(file, "{v}")?;
            } else {
                write!(file, "{v},")?;
            }
        }
    }
    Ok(())
}
