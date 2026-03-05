use raylib::prelude::*;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process;
use std::time::Instant;

use crate::bodies::Bodies;
use crate::config::{DRAW_BUDGET, WINDOW_CENTER};
use crate::render::draw;
use crate::sim::step;

pub struct BenchmarkConfig {
    pub frames: usize,
    pub warmup_frames: usize,
    pub output_path: String,
}

#[derive(Clone, Copy)]
struct FrameSample {
    step_ms: f64,
    draw_ms: f64,
}

pub fn parse_benchmark_config() -> Option<BenchmarkConfig> {
    let mut benchmark_enabled = false;
    let mut frames = 600usize;
    let mut warmup_frames = 120usize;
    let mut output_path = String::from("perf_samples.csv");

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--benchmark" => benchmark_enabled = true,
            "--frames" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --frames");
                    process::exit(2);
                });
                frames = value.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("invalid --frames value: {value}");
                    process::exit(2);
                });
            }
            "--warmup-frames" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --warmup-frames");
                    process::exit(2);
                });
                warmup_frames = value.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("invalid --warmup-frames value: {value}");
                    process::exit(2);
                });
            }
            "--output" => {
                output_path = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --output");
                    process::exit(2);
                });
            }
            _ => {}
        }
    }

    if benchmark_enabled {
        Some(BenchmarkConfig {
            frames,
            warmup_frames,
            output_path,
        })
    } else {
        None
    }
}

pub fn run_benchmark(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    bodies: &mut Bodies,
    texture: &mut RenderTexture2D,
    config: &BenchmarkConfig,
) {
    let mut draw_offset = 0usize;
    let mouse_pos = WINDOW_CENTER;
    let mut samples = Vec::with_capacity(config.frames);

    for _ in 0..config.warmup_frames {
        if rl.window_should_close() {
            break;
        }
        step(bodies, mouse_pos);
        draw(
            rl,
            thread,
            &bodies.pos,
            texture,
            draw_offset,
            mouse_pos,
            "",
            false,
        );
        draw_offset = (draw_offset + DRAW_BUDGET) % bodies.pos.len();
    }

    for _ in 0..config.frames {
        if rl.window_should_close() {
            break;
        }

        let step_start = Instant::now();
        step(bodies, mouse_pos);
        let step_ms = step_start.elapsed().as_secs_f64() * 1000.0;

        let draw_start = Instant::now();
        draw(
            rl,
            thread,
            &bodies.pos,
            texture,
            draw_offset,
            mouse_pos,
            "",
            false,
        );
        let draw_ms = draw_start.elapsed().as_secs_f64() * 1000.0;

        samples.push(FrameSample { step_ms, draw_ms });
        draw_offset = (draw_offset + DRAW_BUDGET) % bodies.pos.len();
    }

    match write_benchmark_csv(&config.output_path, &samples) {
        Ok(()) => {
            if samples.is_empty() {
                println!("benchmark completed with no captured samples");
                return;
            }

            let len = samples.len() as f64;
            let avg_step_ms = samples.iter().map(|s| s.step_ms).sum::<f64>() / len;
            let avg_draw_ms = samples.iter().map(|s| s.draw_ms).sum::<f64>() / len;
            let avg_total_ms = avg_step_ms + avg_draw_ms;

            let mut step_values = samples.iter().map(|s| s.step_ms).collect::<Vec<_>>();
            let mut draw_values = samples.iter().map(|s| s.draw_ms).collect::<Vec<_>>();
            let mut total_values = samples
                .iter()
                .map(|s| s.step_ms + s.draw_ms)
                .collect::<Vec<_>>();

            let p95_step = percentile(&mut step_values, 0.95);
            let p95_draw = percentile(&mut draw_values, 0.95);
            let p95_total = percentile(&mut total_values, 0.95);

            println!(
                "benchmark done: frames={} warmup={} output={}",
                samples.len(),
                config.warmup_frames,
                config.output_path
            );
            println!(
                "avg step={:.3}ms draw={:.3}ms total={:.3}ms (~{:.1} fps)",
                avg_step_ms,
                avg_draw_ms,
                avg_total_ms,
                1000.0 / avg_total_ms
            );
            println!(
                "p95 step={:.3}ms draw={:.3}ms total={:.3}ms",
                p95_step, p95_draw, p95_total
            );
        }
        Err(err) => eprintln!(
            "failed to write benchmark csv to {}: {err}",
            config.output_path
        ),
    }
}

fn write_benchmark_csv(path: &str, samples: &[FrameSample]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "frame,step_ms,draw_ms,total_ms,step_ratio,draw_ratio"
    )?;
    for (i, sample) in samples.iter().enumerate() {
        let total = sample.step_ms + sample.draw_ms;
        let step_ratio = if total > 0.0 {
            sample.step_ms / total
        } else {
            0.0
        };
        let draw_ratio = if total > 0.0 {
            sample.draw_ms / total
        } else {
            0.0
        };
        writeln!(
            writer,
            "{},{:.6},{:.6},{:.6},{:.6},{:.6}",
            i, sample.step_ms, sample.draw_ms, total, step_ratio, draw_ratio
        )?;
    }
    writer.flush()
}

fn percentile(values: &mut [f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rank = ((values.len() - 1) as f64 * p).round() as usize;
    values[rank]
}
