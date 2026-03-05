use std::time::{Duration, Instant};

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::config::{
    BLOCK_ON_GPU_EACH_FRAME, CHAIN_FIXED_SPEED, CHAIN_USE_GRID_SPAWN, ENABLE_BOUNDS, NUM_PARTICLES,
    WORKGROUP_SIZE,
};
use crate::types::{make_chain_particles, SimParams};

const CHAIN_COMPUTE_SHADER: &str = include_str!("../shaders/chain_compute.wgsl");
const RENDER_SHADER: &str = include_str!("../shaders/render.wgsl");

pub struct ChainGpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,

    _particle_buffers: [wgpu::Buffer; 2],
    params_buffer: wgpu::Buffer,
    compute_bind_groups: [wgpu::BindGroup; 2],
    render_bind_groups: [wgpu::BindGroup; 2],
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,

    current_read_idx: usize,
    frame_counter: u32,
    stats_window_start: Instant,
}

impl ChainGpuState {
    pub async fn new(window: &'static winit::window::Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .expect("failed to create wgpu surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("no suitable GPU adapters found");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("chain_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let present_mode = if surface_caps
            .present_modes
            .contains(&wgpu::PresentMode::Immediate)
        {
            wgpu::PresentMode::Immediate
        } else {
            wgpu::PresentMode::Fifo
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let particles = make_chain_particles(
            NUM_PARTICLES,
            config.width,
            config.height,
            CHAIN_USE_GRID_SPAWN,
        );
        let particle_a = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chain_particle_a"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let particle_b = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chain_particle_b"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let params = SimParams {
            target_window: [
                config.width as f32 * 0.5,
                config.height as f32 * 0.5,
                config.width as f32,
                config.height as f32,
            ],
            sim: [
                CHAIN_FIXED_SPEED,
                if ENABLE_BOUNDS { 1.0 } else { 0.0 },
                NUM_PARTICLES as f32,
                0.0,
            ],
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chain_params_buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("chain_compute_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let render_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("chain_render_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let compute_bind_group_ab = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chain_compute_bg_ab"),
            layout: &compute_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_bind_group_ba = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chain_compute_bg_ba"),
            layout: &compute_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: particle_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let render_bind_group_a = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chain_render_bg_a"),
            layout: &render_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_a.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let render_bind_group_b = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("chain_render_bg_b"),
            layout: &render_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_b.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chain_compute_shader"),
            source: wgpu::ShaderSource::Wgsl(CHAIN_COMPUTE_SHADER.into()),
        });

        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chain_render_shader"),
            source: wgpu::ShaderSource::Wgsl(RENDER_SHADER.into()),
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("chain_compute_pipeline_layout"),
                bind_group_layouts: &[&compute_bgl],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("chain_compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "cs_main",
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("chain_render_pipeline_layout"),
                bind_group_layouts: &[&render_bgl],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chain_render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            _particle_buffers: [particle_a, particle_b],
            params_buffer,
            compute_bind_groups: [compute_bind_group_ab, compute_bind_group_ba],
            render_bind_groups: [render_bind_group_a, render_bind_group_b],
            compute_pipeline,
            render_pipeline,
            current_read_idx: 0,
            frame_counter: 0,
            stats_window_start: Instant::now(),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.write_uniform_params();
    }

    pub fn recover_surface(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        self.write_uniform_params();

        let frame = self.surface.get_current_texture()?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("chain_frame_encoder"),
            });

        let write_idx = 1_usize - self.current_read_idx;

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("chain_compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_groups[self.current_read_idx], &[]);
            let workgroups = NUM_PARTICLES.div_ceil(WORKGROUP_SIZE);
            compute_pass.dispatch_workgroups(workgroups, 1, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("chain_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_groups[write_idx], &[]);
            render_pass.draw(0..NUM_PARTICLES, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        if BLOCK_ON_GPU_EACH_FRAME {
            let _ = self.device.poll(wgpu::Maintain::Wait);
        }
        frame.present();

        self.current_read_idx = write_idx;
        self.frame_counter += 1;

        let elapsed = self.stats_window_start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_counter as f64 / elapsed.as_secs_f64();
            window.set_title(&format!(
                "chain gpu | {} particles | {:.1} FPS",
                NUM_PARTICLES, fps
            ));
            println!(
                "chain gpu stats: particles={} fps={:.1} frame_ms={:.3}",
                NUM_PARTICLES,
                fps,
                1000.0 / fps.max(0.1)
            );
            self.frame_counter = 0;
            self.stats_window_start = Instant::now();
        }

        Ok(())
    }

    fn write_uniform_params(&mut self) {
        let params = SimParams {
            target_window: [
                self.config.width as f32 * 0.5,
                self.config.height as f32 * 0.5,
                self.config.width as f32,
                self.config.height as f32,
            ],
            sim: [
                CHAIN_FIXED_SPEED,
                if ENABLE_BOUNDS { 1.0 } else { 0.0 },
                NUM_PARTICLES as f32,
                0.0,
            ],
        };
        self.queue
            .write_buffer(&self.params_buffer, 0, bytemuck::bytes_of(&params));
    }
}
