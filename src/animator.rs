use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::cell::Grid;

#[derive(Clone, Debug)]
pub enum Effect {
    DissolveIn,
    DissolveOut,
    SwirlIn,
    SwirlOut,
    WhirlIn,
    WhirlOut,
    KenBurns,
}

impl Effect {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "dissolve-in" => Some(Self::DissolveIn),
            "dissolve-out" => Some(Self::DissolveOut),
            "swirl-in" => Some(Self::SwirlIn),
            "swirl-out" => Some(Self::SwirlOut),
            "whirl-in" => Some(Self::WhirlIn),
            "whirl-out" => Some(Self::WhirlOut),
            "ken-burns" => Some(Self::KenBurns),
            _ => None,
        }
    }
}

pub struct Animator {
    grid: Grid,
    effect: Effect,
    duration: Duration,
}

impl Animator {
    pub fn new(grid: Grid, effect: Effect, duration_secs: f64) -> Self {
        Self {
            grid,
            effect,
            duration: Duration::from_secs_f64(duration_secs),
        }
    }

    pub fn play(&self) -> io::Result<()> {
        let mut out = io::stdout();
        write!(out, "\x1b[?25l\x1b[2J")?;
        out.flush()?;

        match self.effect {
            Effect::DissolveIn => self.play_dissolve_in(&mut out)?,
            Effect::DissolveOut => self.play_dissolve_out(&mut out)?,
            Effect::SwirlIn => self.play_swirl(&mut out, false)?,
            Effect::SwirlOut => self.play_swirl(&mut out, true)?,
            Effect::WhirlIn => self.play_whirl(&mut out, false)?,
            Effect::WhirlOut => self.play_whirl(&mut out, true)?,
            Effect::KenBurns => self.play_ken_burns(&mut out)?,
        }

        write!(out, "\x1b[?25h")?;
        out.flush()?;
        Ok(())
    }

    fn all_positions(&self) -> Vec<(usize, usize)> {
        let mut positions = Vec::new();
        for (r, row) in self.grid.iter().enumerate() {
            for (c, _) in row.iter().enumerate() {
                positions.push((r, c));
            }
        }
        positions
    }

    fn draw_cell(&self, out: &mut impl Write, row: usize, col: usize) -> io::Result<()> {
        let cell = &self.grid[row][col];
        write!(
            out,
            "\x1b[{};{}H{}{}{}",
            row + 1,
            col + 1,
            cell.color_pre,
            cell.ch,
            cell.color_suf
        )
    }

    fn clear_cell(&self, out: &mut impl Write, row: usize, col: usize) -> io::Result<()> {
        write!(out, "\x1b[{};{}H ", row + 1, col + 1)
    }

    fn draw_full(&self, out: &mut impl Write) -> io::Result<()> {
        for (r, row) in self.grid.iter().enumerate() {
            for (c, _) in row.iter().enumerate() {
                self.draw_cell(out, r, c)?;
            }
        }
        out.flush()
    }

    fn play_dissolve_in(&self, out: &mut impl Write) -> io::Result<()> {
        let mut positions = self.all_positions();
        positions.shuffle(&mut thread_rng());
        self.play_batched(out, &positions, true)
    }

    fn play_dissolve_out(&self, out: &mut impl Write) -> io::Result<()> {
        self.draw_full(out)?;
        thread::sleep(Duration::from_millis(500));
        let mut positions = self.all_positions();
        positions.shuffle(&mut thread_rng());
        self.play_batched(out, &positions, false)?;
        thread::sleep(Duration::from_millis(300));
        self.draw_full(out)?;
        Ok(())
    }

    fn play_batched(
        &self,
        out: &mut impl Write,
        positions: &[(usize, usize)],
        reveal: bool,
    ) -> io::Result<()> {
        let total = positions.len();
        if total == 0 {
            return Ok(());
        }
        let target_fps = 30.0;
        let total_frames = (self.duration.as_secs_f64() * target_fps) as usize;
        let cells_per_frame = (total as f64 / total_frames.max(1) as f64).ceil() as usize;
        let frame_duration = self.duration / total_frames.max(1) as u32;
        let start = Instant::now();

        for (i, chunk) in positions.chunks(cells_per_frame.max(1)).enumerate() {
            for &(r, c) in chunk {
                if reveal {
                    self.draw_cell(out, r, c)?;
                } else {
                    self.clear_cell(out, r, c)?;
                }
            }
            out.flush()?;
            let target_time = frame_duration * (i + 1) as u32;
            let elapsed = start.elapsed();
            if elapsed < target_time {
                thread::sleep(target_time - elapsed);
            }
        }

        if reveal {
            self.draw_full(out)?;
        }
        Ok(())
    }

    fn spiral_positions(&self) -> Vec<(usize, usize)> {
        let rows = self.grid.len();
        if rows == 0 {
            return Vec::new();
        }
        let cols = self.grid[0].len();
        if cols == 0 {
            return Vec::new();
        }

        let mut positions = Vec::with_capacity(rows * cols);
        let (mut top, mut bottom, mut left, mut right) =
            (0i32, rows as i32 - 1, 0i32, cols as i32 - 1);

        while top <= bottom && left <= right {
            for c in left..=right {
                positions.push((top as usize, c as usize));
            }
            top += 1;
            for r in top..=bottom {
                positions.push((r as usize, right as usize));
            }
            right -= 1;
            if top <= bottom {
                for c in (left..=right).rev() {
                    positions.push((bottom as usize, c as usize));
                }
                bottom -= 1;
            }
            if left <= right {
                for r in (top..=bottom).rev() {
                    positions.push((r as usize, left as usize));
                }
                left += 1;
            }
        }
        positions
    }

    fn play_swirl(&self, out: &mut impl Write, from_center: bool) -> io::Result<()> {
        let mut positions = self.spiral_positions();
        if from_center {
            positions.reverse();
        }
        self.play_batched(out, &positions, true)
    }

    fn whirl_positions(&self) -> Vec<(usize, usize)> {
        let rows = self.grid.len();
        if rows == 0 {
            return Vec::new();
        }
        let cols = self.grid[0].len();
        if cols == 0 {
            return Vec::new();
        }

        let center_r = rows as f64 / 2.0;
        let center_c = cols as f64 / 2.0;
        let max_radius = (center_r * center_r + center_c * center_c).sqrt();
        // Number of full rotations from center to edge
        let num_rotations = 4.0;

        let mut positions: Vec<(f64, usize, usize)> = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for c in 0..cols {
                let dr = r as f64 - center_r;
                let dc = c as f64 - center_c;
                let radius = (dr * dr + dc * dc).sqrt();
                let angle = dr.atan2(dc); // -PI..PI
                let normalized_radius = radius / max_radius;
                // Angle normalized to 0..1
                let angle_norm = (angle + std::f64::consts::PI) / (2.0 * std::f64::consts::PI);
                // Radius is the primary sort; angle creates a spin within each ring band.
                // Each 1/num_rotations band of radius gets one full angular sweep.
                let key = normalized_radius + angle_norm / num_rotations;
                positions.push((key, r, c));
            }
        }
        positions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        positions.into_iter().map(|(_, r, c)| (r, c)).collect()
    }

    fn play_whirl(&self, out: &mut impl Write, outward: bool) -> io::Result<()> {
        let mut positions = self.whirl_positions();
        if !outward {
            // whirl-in: edge first, center last
            positions.reverse();
        }
        self.play_batched(out, &positions, true)
    }

    fn play_ken_burns(&self, out: &mut impl Write) -> io::Result<()> {
        let rows = self.grid.len();
        if rows == 0 {
            return Ok(());
        }
        let cols = self.grid[0].len();

        if rows <= 1 || cols <= 1 {
            self.draw_full(out)?;
            thread::sleep(self.duration);
            return Ok(());
        }

        // Viewport is ~70% of the grid; pan across the remaining 30%
        let view_h = (rows as f64 * 0.7).ceil() as usize;
        let view_w = (cols as f64 * 0.7).ceil() as usize;
        let max_y = rows.saturating_sub(view_h);
        let max_x = cols.saturating_sub(view_w);

        // If grid is too small for a meaningful pan, just show it
        if max_y == 0 && max_x == 0 {
            self.draw_full(out)?;
            thread::sleep(self.duration);
            return Ok(());
        }

        let target_fps = 24.0;
        let total_frames = (self.duration.as_secs_f64() * target_fps) as usize;
        let frame_duration = self.duration / total_frames.max(1) as u32;
        let start = Instant::now();

        for frame in 0..total_frames {
            let t = frame as f64 / total_frames.max(1) as f64;

            let view_y = (max_y as f64 * t) as usize;
            let view_x = (max_x as f64 * t) as usize;

            write!(out, "\x1b[H")?;
            for r in 0..view_h.min(rows - view_y) {
                for c in 0..view_w.min(cols - view_x) {
                    let cell = &self.grid[view_y + r][view_x + c];
                    write!(out, "{}{}{}", cell.color_pre, cell.ch, cell.color_suf)?;
                }
                // Clear remainder of terminal line
                write!(out, "\x1b[K")?;
                if r < view_h - 1 {
                    writeln!(out)?;
                }
            }
            out.flush()?;

            let target_time = frame_duration * (frame + 1) as u32;
            let elapsed = start.elapsed();
            if elapsed < target_time {
                thread::sleep(target_time - elapsed);
            }
        }

        // Final frame: show full image
        write!(out, "\x1b[2J")?;
        self.draw_full(out)?;
        Ok(())
    }
}
