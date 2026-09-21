//! Run with `cargo run -p egui_glass_demo --example live_benchmark --release`.
//! Headless timings include CPU work, submission and waiting for GPU completion, not presentation.
#[path = "support/headless.rs"]
mod headless;

use egui_glass::LiveBackdropQuality::{Balanced, Full, Performance};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gpu = pollster::block_on(headless::Harness::new())?;
    for (size, ppp) in [([801, 603], 1.25), ([1280, 720], 1.0), ([2560, 1440], 2.0)] {
        gpu.resize(size, ppp);
        for quality in [Some(Full), Some(Performance), None, Some(Balanced), Some(Full)] {
            gpu.frame(quality, 1, true)?;
            gpu.verify_landmarks()?;
        }
    }
    println!("GPU landmark checks passed (quality changes, resize, odd sizes, HiDPI).");
    println!("size,ppp,glass,quality,layout_median_ms,frame_median_ms,frame_p95_ms");
    for (size, ppp) in [([1280, 720], 1.0), ([2560, 1440], 2.0)] {
        gpu.resize(size, ppp);
        for glass in [1, 16] {
            for quality in [None, Some(Full), Some(Balanced), Some(Performance)] {
                for _ in 0..20 { gpu.frame(quality, glass, false)?; }
                let mut layout = Vec::new();
                let mut total = Vec::new();
                for _ in 0..100 {
                    let [cpu, frame] = gpu.frame(quality, glass, false)?;
                    layout.push(cpu);
                    total.push(frame);
                }
                layout.sort_by(f64::total_cmp);
                total.sort_by(f64::total_cmp);
                println!("{}x{},{ppp},{glass},{quality:?},{:.3},{:.3},{:.3}", size[0], size[1], layout[49], total[49], total[94]);
            }
        }
    }
    Ok(())
}
