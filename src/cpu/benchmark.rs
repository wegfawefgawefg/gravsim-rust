use raylib::prelude::*;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process;
use std::time::Instant;

use crate::bodies::Bodies;
use crate::config::{DEFAULT_STEP_CHUNK_SIZE, DRAW_BUDGET, WINDOW_CENTER};
use crate::render::{render_mode_label, RenderMode, Renderer};
use crate::sim::{
    step_kernel_label, step_with_kernel, step_with_kernel_collect_draw_indices, DrawSelection,
    StepKernel,
};

#[derive(Clone, Copy, Debug)]
pub enum BenchmarkMode {
    Full,
    StepOnly,
    DrawOnly,
}

pub struct BenchmarkConfig {
    pub frames: usize,
    pub warmup_frames: usize,
    pub output_path: String,
    pub mode: BenchmarkMode,
    pub step_kernel: StepKernel,
    pub render_mode: RenderMode,
    pub fused_step_draw: bool,
}

#[derive(Clone, Copy)]
struct FrameSample {
    step_ms: f64,
    draw_ms: f64,
}

pub fn parse_benchmark_config() -> Option<BenchmarkConfig> {
    let mut benchmark_enabled = false;
    let mut mode = BenchmarkMode::Full;
    let mut frames = 600usize;
    let mut warmup_frames = 120usize;
    let mut output_path = String::from("perf_samples.csv");
    let mut step_kernel_name = String::from("zip");
    let mut chunk_size = DEFAULT_STEP_CHUNK_SIZE;
    let mut render_mode = RenderMode::Rgba;
    let mut fused_step_draw = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--benchmark" => benchmark_enabled = true,
            "--benchmark-step-only" => {
                benchmark_enabled = true;
                mode = BenchmarkMode::StepOnly;
            }
            "--benchmark-draw-only" => {
                benchmark_enabled = true;
                mode = BenchmarkMode::DrawOnly;
            }
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
            "--step-kernel" => {
                step_kernel_name = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --step-kernel (expected zip|chunked)");
                    process::exit(2);
                });
            }
            "--chunk-size" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --chunk-size");
                    process::exit(2);
                });
                chunk_size = value.parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("invalid --chunk-size value: {value}");
                    process::exit(2);
                });
            }
            "--render-mode" => {
                let value = args.next().unwrap_or_else(|| {
                    eprintln!("missing value for --render-mode (expected rgba|bitset)");
                    process::exit(2);
                });
                render_mode = match value.as_str() {
                    "rgba" => RenderMode::Rgba,
                    "bitset" => RenderMode::Bitset,
                    _ => {
                        eprintln!("invalid --render-mode value: {value} (expected rgba|bitset)");
                        process::exit(2);
                    }
                };
            }
            "--fused-step-draw" => fused_step_draw = true,
            _ => {}
        }
    }

    if !benchmark_enabled {
        return None;
    }

    let step_kernel = match step_kernel_name.as_str() {
        "zip" => StepKernel::Zip,
        "chunked" => StepKernel::Chunked { chunk_size },
        _ => {
            eprintln!("invalid --step-kernel value: {step_kernel_name} (expected zip|chunked)");
            process::exit(2);
        }
    };

    Some(BenchmarkConfig {
        frames,
        warmup_frames,
        output_path,
        mode,
        step_kernel,
        render_mode,
        fused_step_draw,
    })
}

pub fn run_benchmark(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    bodies: &mut Bodies,
    renderer: &mut Renderer,
    config: &BenchmarkConfig,
) {
    let mut draw_offset = 0usize;
    let mouse_pos = WINDOW_CENTER;
    let mut samples = Vec::with_capacity(config.frames);

    for _ in 0..config.warmup_frames {
        if rl.window_should_close() {
            break;
        }

        run_benchmark_frame(rl, thread, bodies, renderer, draw_offset, mouse_pos, config);
        if !matches!(config.mode, BenchmarkMode::StepOnly) {
            draw_offset = (draw_offset + DRAW_BUDGET) % bodies.pos.len();
        }
    }

    for _ in 0..config.frames {
        if rl.window_should_close() {
            break;
        }

        let sample =
            run_benchmark_frame(rl, thread, bodies, renderer, draw_offset, mouse_pos, config);
        if !matches!(config.mode, BenchmarkMode::StepOnly) {
            draw_offset = (draw_offset + DRAW_BUDGET) % bodies.pos.len();
        }
        samples.push(sample);
    }

    match write_benchmark_csv(&config.output_path, &samples, config) {
        Ok(()) => print_summary(&samples, config),
        Err(err) => eprintln!(
            "failed to write benchmark csv to {}: {err}",
            config.output_path
        ),
    }
}

fn run_benchmark_frame(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    bodies: &mut Bodies,
    renderer: &mut Renderer,
    draw_offset: usize,
    mouse_pos: glam::Vec2,
    config: &BenchmarkConfig,
) -> FrameSample {
    let mut step_ms = 0.0;
    let mut draw_ms = 0.0;

    if !matches!(config.mode, BenchmarkMode::DrawOnly) {
        let step_start = Instant::now();
        if matches!(config.mode, BenchmarkMode::Full) && config.fused_step_draw {
            let draw_selection = DrawSelection {
                draw_offset,
                draw_budget: DRAW_BUDGET.min(bodies.pos.len()),
                total_bodies: bodies.pos.len(),
            };
            let draw_indices = step_with_kernel_collect_draw_indices(
                bodies,
                mouse_pos,
                config.step_kernel,
                draw_selection,
            );
            step_ms = step_start.elapsed().as_secs_f64() * 1000.0;

            let draw_start = Instant::now();
            renderer.draw_indices(
                rl,
                thread,
                &draw_indices,
                mouse_pos,
                "",
                false,
                bodies.pos.len(),
            );
            draw_ms = draw_start.elapsed().as_secs_f64() * 1000.0;
            return FrameSample { step_ms, draw_ms };
        }

        step_with_kernel(bodies, mouse_pos, config.step_kernel);
        step_ms = step_start.elapsed().as_secs_f64() * 1000.0;
    }

    if !matches!(config.mode, BenchmarkMode::StepOnly) {
        let draw_start = Instant::now();
        renderer.draw_positions(rl, thread, &bodies.pos, draw_offset, mouse_pos, "", false);
        draw_ms = draw_start.elapsed().as_secs_f64() * 1000.0;
    }

    FrameSample { step_ms, draw_ms }
}

fn write_benchmark_csv(
    path: &str,
    samples: &[FrameSample],
    config: &BenchmarkConfig,
) -> std::io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    writeln!(
        writer,
        "frame,mode,render_mode,fused_step_draw,step_kernel,step_ms,draw_ms,total_ms,step_ratio,draw_ratio"
    )?;

    let mode = benchmark_mode_label(config.mode);
    let render_mode = render_mode_label(config.render_mode);
    let fused_step_draw = if config.fused_step_draw { 1 } else { 0 };
    let step_kernel = step_kernel_label(config.step_kernel);

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
            "{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6}",
            i,
            mode,
            render_mode,
            fused_step_draw,
            step_kernel,
            sample.step_ms,
            sample.draw_ms,
            total,
            step_ratio,
            draw_ratio
        )?;
    }

    writer.flush()
}

fn print_summary(samples: &[FrameSample], config: &BenchmarkConfig) {
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
        "benchmark done: mode={} render_mode={} fused_step_draw={} step_kernel={} frames={} warmup={} output={}",
        benchmark_mode_label(config.mode),
        render_mode_label(config.render_mode),
        if config.fused_step_draw { "on" } else { "off" },
        step_kernel_label(config.step_kernel),
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

fn benchmark_mode_label(mode: BenchmarkMode) -> &'static str {
    match mode {
        BenchmarkMode::Full => "full",
        BenchmarkMode::StepOnly => "step_only",
        BenchmarkMode::DrawOnly => "draw_only",
    }
}

fn percentile(values: &mut [f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rank = ((values.len() - 1) as f64 * p).round() as usize;
    values[rank]
}
