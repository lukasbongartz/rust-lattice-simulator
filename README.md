# Phase Simulation (Rust + Macroquad)

A fast, interactive 2D lattice phase-transition simulator written in Rust. It visualizes a simple lattice gas/Ising-like model with controls for temperature and chemical potential, and renders an equilibrium phase diagram.

- Interactive UI with real-time simulation
- Phase diagram and mean-field free-energy plot
- CSV export of time series and lattice snapshots
- GPU-friendly rendering via `macroquad`

![Screenshot](screenshot.png)

## Getting Started

### Web Version (No Installation Required)

The simulation is available as a WebAssembly app. See the `web-deploy/` directory for deployment-ready files.

**Local testing:**
```bash
cd web-deploy
python3 -m http.server 8000
# Open http://localhost:8000
```

### Desktop Version

**Prerequisites:**
- Rust (stable). Install via `rustup`.
- macOS/Linux/Windows supported.

**Build and run:**
```bash
cargo run
```

**Build for web:**
```bash
cargo build --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/phase_simulation.wasm web-deploy/
```

### Controls

- **↑/↓** - Adjust Temperature
- **←/→** - Adjust Chemical Potential
- **Space** - Randomize Grid
- **M** - Cycle display modes (UI → Phase diagram → Free-energy plot)
- **D** - Toggle density popup
- **S** - Save CSV snapshot

## Repository Layout

- `src/main.rs` - Interactive GUI app (Macroquad)
- `src/density_plot.rs` - Lightweight time-series popup
- `web-deploy/` - Production-ready web deployment files
- `.cargo/config.toml` - WASM build configuration
 

## How It Works

- Lattice gas with nearest-neighbor interaction `J`
- Glauber dynamics Monte Carlo updates
- Mean-field functional:

  $$
  f_{tc}(\rho) = \frac{2J\rho^2 + \mu\rho}{T} - \left[ \rho\ln\rho + (1-\rho)\ln(1-\rho) \right]
  $$

## Data Export

Time-series CSV schema:
```
step,temperature,chem_potential,density
```
Snapshots are matrices of `0/1` (empty/molecule).

## License

MIT.