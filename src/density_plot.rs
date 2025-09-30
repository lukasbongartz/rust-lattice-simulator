use macroquad::prelude::*;

pub struct TimeSeriesRingBuffer {
    buffer: Vec<f32>,
    capacity: usize,
    head: usize,
    len: usize,
}

impl TimeSeriesRingBuffer {
    pub fn with_capacity(capacity: usize) -> Self {
        Self { buffer: vec![0.0; capacity], capacity, head: 0, len: 0 }
    }

    pub fn push(&mut self, value: f32) {
        if self.capacity == 0 { return; }
        self.buffer[self.head] = value;
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity { self.len += 1; }
    }

    pub fn iter_in_order(&self) -> impl Iterator<Item = f32> + '_ {
        let start = (self.head + self.capacity - self.len) % self.capacity;
        (0..self.len).map(move |i| {
            let idx = (start + i) % self.capacity;
            self.buffer[idx]
        })
    }
}

pub struct DensityPopup {
    series: TimeSeriesRingBuffer,
    is_open: bool,
    y_min_seen: f32,
    y_max_seen: f32,
}

impl DensityPopup {
    pub fn new(capacity: usize) -> Self {
        Self { series: TimeSeriesRingBuffer::with_capacity(capacity), is_open: false, y_min_seen: 1.0, y_max_seen: 0.0 }
    }

    pub fn toggle(&mut self) { self.is_open = !self.is_open; }

    pub fn record_density(&mut self, density: f32) {
        self.series.push(density);
        if density < self.y_min_seen { self.y_min_seen = density; }
        if density > self.y_max_seen { self.y_max_seen = density; }
    }

    pub fn draw(&self, rect: Rect) {
        if !self.is_open { return; }
        let bg = Color::new(0.05, 0.05, 0.05, 0.95);
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
        draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, GRAY);

        let padding = 12.0;
        let plot_rect = Rect { x: rect.x + padding, y: rect.y + padding, w: rect.w - 2.0 * padding, h: rect.h - 2.0 * padding };

        // Axes
        draw_line(plot_rect.x, plot_rect.y + plot_rect.h, plot_rect.x + plot_rect.w, plot_rect.y + plot_rect.h, 1.0, LIGHTGRAY);
        draw_line(plot_rect.x, plot_rect.y, plot_rect.x, plot_rect.y + plot_rect.h, 1.0, LIGHTGRAY);

        // Determine y-range with small padding
        let mut ymin = 0.0_f32;
        let mut ymax = 1.0_f32;
        if self.series.len > 0 {
            ymin = self.y_min_seen;
            ymax = self.y_max_seen;
            if (ymax - ymin) < 0.02 {
                let mid = 0.5 * (ymin + ymax);
                ymin = (mid - 0.01).max(0.0);
                ymax = (mid + 0.01).min(1.0);
            }
        }

        // Draw the polyline
        let n = self.series.len;
        if n >= 2 {
            let step_x = plot_rect.w / (n as f32 - 1.0).max(1.0);
            let mut prev_x = plot_rect.x;
            let mut prev_y = plot_rect.y + plot_rect.h - ((self.series.iter_in_order().next().unwrap_or(0.0) - ymin) / (ymax - ymin).max(1e-6)) * plot_rect.h;
            for (i, yv) in self.series.iter_in_order().enumerate() {
                if i == 0 { continue; }
                let x = plot_rect.x + i as f32 * step_x;
                let y = plot_rect.y + plot_rect.h - ((yv - ymin) / (ymax - ymin).max(1e-6)) * plot_rect.h;
                draw_line(prev_x, prev_y, x, y, 2.0, YELLOW);
                prev_x = x;
                prev_y = y;
            }
        }

        let label = TextParams { font_size: 22, color: YELLOW, ..Default::default() };
        draw_text_ex("Density vs Time", rect.x + 16.0, rect.y + 28.0, label);
    }
}


