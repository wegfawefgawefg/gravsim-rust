use std::time::{Duration, Instant};

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::config::{
    gravity_target, pick_present_mode, BLOCK_ON_GPU_EACH_FRAME, DEFAULT_FADE_ENABLED,
    ENABLE_BOUNDS, FADE_ALPHA, G, NUM_PARTICLES, POINT_ALPHA, WORKGROUP_SIZE,
};
use crate::types::{make_particles, Particle, SimParams};

const COMPUTE_SHADER: &str = include_str!("../shaders/compute.wgsl");
const RENDER_SHADER: &str = include_str!("../shaders/render.wgsl");
const FADE_SHADER: &str = include_str!("../shaders/fade_overlay.wgsl");
const BLIT_SHADER: &str = include_str!("../shaders/blit_texture.wgsl");
const MAX_DISPATCH_GROUPS_PER_DIM: u32 = 65_535;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FadeParams {
    rgba: [f32; 4],
}

struct ParticleShard {
    count: u32,
    _buffer: wgpu::Buffer,
    params_buffer: wgpu::Buffer,
    compute_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup,
}

pub struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,

    total_particle_count: u64,
    particles_per_shard_max: u32,
    supports_shader_f16: bool,
    present_mode: wgpu::PresentMode,
    shards: Vec<ParticleShard>,
    compute_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,

    fade_enabled: bool,
    trail_initialized: bool,
    _trail_texture: wgpu::Texture,
    trail_view: wgpu::TextureView,
    _fade_params_buffer: wgpu::Buffer,
    fade_bind_group: wgpu::BindGroup,
    fade_pipeline: wgpu::RenderPipeline,
    blit_sampler: wgpu::Sampler,
    blit_bind_group_layout: wgpu::BindGroupLayout,
    blit_bind_group: wgpu::BindGroup,
    blit_pipeline: wgpu::RenderPipeline,

    mouse_pos: [f32; 2],
    frame_counter: u32,
    stats_window_start: Instant,
}

impl GpuState {
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

        let adapter_features = adapter.features();
        let supports_shader_f16 = adapter_features.contains(wgpu::Features::SHADER_F16);
        let limits = adapter.limits();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: limits.clone(),
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
        let present_mode = pick_present_mode(&surface_caps);

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

        let compute_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("compute_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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
            label: Some("render_bgl"),
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

        let particles_per_shard_max = max_particles_per_shard(&limits);
        if particles_per_shard_max == 0 {
            panic!("device limits are too small for even one particle element");
        }
        let requested_particles = NUM_PARTICLES as u64;
        let shards = create_particle_shards(
            &device,
            &compute_bgl,
            &render_bgl,
            requested_particles,
            particles_per_shard_max,
            config.width,
            config.height,
        );
        let total_particle_count = shards.iter().map(|s| s.count as u64).sum::<u64>();

        println!(
            "gpu caps: shader_f16={} max_storage_binding_bytes={} max_buffer_bytes={} max_particles_per_shard={} requested_particles={} shard_count={} allocated_particles={}",
            supports_shader_f16,
            limits.max_storage_buffer_binding_size,
            limits.max_buffer_size,
            particles_per_shard_max,
            requested_particles,
            shards.len(),
            total_particle_count
        );

        let fade_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fade_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blit_bgl"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let fade_params = FadeParams {
            rgba: [0.0, 0.0, 0.0, FADE_ALPHA],
        };
        let fade_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("fade_params_buffer"),
            contents: bytemuck::bytes_of(&fade_params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let fade_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fade_bg"),
            layout: &fade_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: fade_params_buffer.as_entire_binding(),
            }],
        });

        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("compute_shader"),
            source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER.into()),
        });
        let render_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("render_shader"),
            source: wgpu::ShaderSource::Wgsl(RENDER_SHADER.into()),
        });
        let fade_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("fade_shader"),
            source: wgpu::ShaderSource::Wgsl(FADE_SHADER.into()),
        });
        let blit_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("blit_shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()),
        });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&compute_bgl],
                push_constant_ranges: &[],
            });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[&render_bgl],
                push_constant_ranges: &[],
            });
        let fade_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fade_pipeline_layout"),
            bind_group_layouts: &[&fade_bgl],
            push_constant_ranges: &[],
        });
        let blit_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("blit_pipeline_layout"),
            bind_group_layouts: &[&blit_bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "cs_main",
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
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
        let fade_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fade_pipeline"),
            layout: Some(&fade_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &fade_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fade_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
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
        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit_pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
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

        let (trail_texture, trail_view) = create_trail_target(&device, &config);
        let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("blit_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let blit_bind_group =
            create_blit_bind_group(&device, &blit_bind_group_layout, &trail_view, &blit_sampler);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            total_particle_count,
            particles_per_shard_max,
            supports_shader_f16,
            present_mode,
            shards,
            compute_pipeline,
            render_pipeline,
            fade_enabled: DEFAULT_FADE_ENABLED,
            trail_initialized: false,
            _trail_texture: trail_texture,
            trail_view,
            _fade_params_buffer: fade_params_buffer,
            fade_bind_group,
            fade_pipeline,
            blit_sampler,
            blit_bind_group_layout,
            blit_bind_group,
            blit_pipeline,
            mouse_pos: [f32_half(size.width), f32_half(size.height)],
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

        let (trail_texture, trail_view) = create_trail_target(&self.device, &self.config);
        self.blit_bind_group = create_blit_bind_group(
            &self.device,
            &self.blit_bind_group_layout,
            &trail_view,
            &self.blit_sampler,
        );
        self._trail_texture = trail_texture;
        self.trail_view = trail_view;
        self.trail_initialized = false;
    }

    pub fn recover_surface(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn set_mouse(&mut self, x: f32, y: f32) {
        self.mouse_pos = [x, y];
    }

    pub fn toggle_fade(&mut self) {
        self.fade_enabled = !self.fade_enabled;
        self.trail_initialized = false;
        println!(
            "particle sim fade {}",
            if self.fade_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        self.write_shard_params();

        let frame = self.surface.get_current_texture()?;
        let surface_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            for shard in &self.shards {
                compute_pass.set_bind_group(0, &shard.compute_bind_group, &[]);
                let (dispatch_x, dispatch_y) = dispatch_dims_for_count(shard.count);
                compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
            }
        }

        {
            let mut trail_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("trail_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.trail_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: if self.fade_enabled && self.trail_initialized {
                            wgpu::LoadOp::Load
                        } else {
                            wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if self.fade_enabled && self.trail_initialized {
                trail_pass.set_pipeline(&self.fade_pipeline);
                trail_pass.set_bind_group(0, &self.fade_bind_group, &[]);
                trail_pass.draw(0..3, 0..1);
            }

            trail_pass.set_pipeline(&self.render_pipeline);
            for shard in &self.shards {
                trail_pass.set_bind_group(0, &shard.render_bind_group, &[]);
                trail_pass.draw(0..shard.count, 0..1);
            }
        }
        self.trail_initialized = true;

        {
            let mut blit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
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
            blit_pass.set_pipeline(&self.blit_pipeline);
            blit_pass.set_bind_group(0, &self.blit_bind_group, &[]);
            blit_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        if BLOCK_ON_GPU_EACH_FRAME {
            let _ = self.device.poll(wgpu::Maintain::Wait);
        }
        frame.present();

        self.frame_counter += 1;
        let elapsed = self.stats_window_start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let fps = self.frame_counter as f64 / elapsed.as_secs_f64();
            let frame_ms = 1000.0 / fps.max(0.1);
            let fade_status = if self.fade_enabled { "on" } else { "off" };
            window.set_title(&format!(
                "gravsim wgpu | {} particles | {} shards | {:.1} FPS | present {:?} | fade {}",
                self.total_particle_count,
                self.shards.len(),
                fps,
                self.present_mode,
                fade_status
            ));
            println!(
                "wgpu stats: particles={} shards={} max_per_shard={} fps={:.1} frame_ms={:.3} present={:?} fade={} shader_f16={}",
                self.total_particle_count,
                self.shards.len(),
                self.particles_per_shard_max,
                fps,
                frame_ms,
                self.present_mode,
                fade_status,
                self.supports_shader_f16
            );
            self.frame_counter = 0;
            self.stats_window_start = Instant::now();
        }

        Ok(())
    }

    fn write_shard_params(&mut self) {
        let target = gravity_target(self.mouse_pos, self.config.width, self.config.height);
        for shard in &self.shards {
            let params = SimParams {
                target_window: [
                    target[0],
                    target[1],
                    self.config.width as f32,
                    self.config.height as f32,
                ],
                sim: [
                    G,
                    if ENABLE_BOUNDS { 1.0 } else { 0.0 },
                    shard.count as f32,
                    POINT_ALPHA,
                ],
            };
            self.queue
                .write_buffer(&shard.params_buffer, 0, bytemuck::bytes_of(&params));
        }
    }
}

fn create_particle_shards(
    device: &wgpu::Device,
    compute_bgl: &wgpu::BindGroupLayout,
    render_bgl: &wgpu::BindGroupLayout,
    requested_particles: u64,
    max_particles_per_shard: u32,
    width: u32,
    height: u32,
) -> Vec<ParticleShard> {
    let mut remaining = requested_particles;
    let mut shards = Vec::new();

    while remaining > 0 {
        let this_count = remaining.min(max_particles_per_shard as u64) as u32;
        let particles = make_particles(this_count, width, height);
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particle_shard_buffer"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let params = SimParams {
            target_window: [
                width as f32 * 0.5,
                height as f32 * 0.5,
                width as f32,
                height as f32,
            ],
            sim: [
                G,
                if ENABLE_BOUNDS { 1.0 } else { 0.0 },
                this_count as f32,
                POINT_ALPHA,
            ],
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("particle_shard_params_buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle_shard_compute_bg"),
            layout: compute_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("particle_shard_render_bg"),
            layout: render_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_buffer.as_entire_binding(),
                },
            ],
        });

        shards.push(ParticleShard {
            count: this_count,
            _buffer: buffer,
            params_buffer,
            compute_bind_group,
            render_bind_group,
        });
        remaining -= this_count as u64;
    }

    shards
}

fn create_trail_target(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("trail_texture"),
        size: wgpu::Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn create_blit_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    trail_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blit_bg"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(trail_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn max_particles_per_shard(limits: &wgpu::Limits) -> u32 {
    let bytes_per_particle = std::mem::size_of::<Particle>() as u64;
    let max_storage_binding_bytes = limits.max_storage_buffer_binding_size as u64;
    let max_buffer_bytes = limits.max_buffer_size;
    let max_bytes = max_storage_binding_bytes.min(max_buffer_bytes);
    let max_particles = max_bytes / bytes_per_particle;
    max_particles.min(u32::MAX as u64) as u32
}

fn dispatch_dims_for_count(count: u32) -> (u32, u32) {
    let total_workgroups = count.div_ceil(WORKGROUP_SIZE);
    if total_workgroups <= MAX_DISPATCH_GROUPS_PER_DIM {
        return (total_workgroups.max(1), 1);
    }

    let dispatch_x = MAX_DISPATCH_GROUPS_PER_DIM;
    let dispatch_y = total_workgroups.div_ceil(dispatch_x);
    if dispatch_y > MAX_DISPATCH_GROUPS_PER_DIM {
        panic!(
            "particle count {} requires dispatch_y={} > max {}; reduce particle count or increase workgroup size",
            count, dispatch_y, MAX_DISPATCH_GROUPS_PER_DIM
        );
    }
    (dispatch_x, dispatch_y.max(1))
}

fn f32_half(v: u32) -> f32 {
    v as f32 * 0.5
}
