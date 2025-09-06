use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, ColorTargetState, ColorWrites,
    CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Face, FilterMode, FragmentState,
    FrontFace, Instance, MultisampleState, Operations, Origin3d, PipelineCompilationOptions,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderStages, Surface, SurfaceConfiguration, SurfaceError,
    TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
    TextureViewDimension, VertexState, include_wgsl,
};
use winit::window::Window;

use crate::palettes::{PALETTE_SIZE, Palette, RGB10A2};

use super::coordinates::WINDOW_SIZE;

pub struct GPUState {
    surface: Surface<'static>,

    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: RenderPipeline,

    streaming_texture: Texture,
    color_map: Texture,
    diffuse_bind_group: BindGroup,
}

impl GPUState {
    /// Size of the Framebuffer in GPU speech
    const TEXTURE_SIZE: Extent3d = Extent3d {
        width: WINDOW_SIZE.0 as u32,
        height: WINDOW_SIZE.1 as u32,
        depth_or_array_layers: 1,
    };

    const COLOR_MAP_SIZE: Extent3d = Extent3d {
        width: PALETTE_SIZE as u32,
        height: 1,
        depth_or_array_layers: 1,
    };

    pub async fn new(window: std::sync::Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = Instance::default();

        let surface: Surface<'static> = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                // i do very little on the GPU (only scale the whole image once per frame)
                // so i think that this setting should be correct.
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                // i only have two buffers, one constant size and the other one changing with the window size
                // so i don't need a lot of allocations and they don't have to be that fast
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = *surface_caps
            .formats
            .iter()
            .find(|f| !f.is_srgb())
            .unwrap_or(&surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2, // default
        };

        surface.configure(&device, &config);

        let window_texture = device.create_texture(&TextureDescriptor {
            size: Self::TEXTURE_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::R8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("window texture"),
            view_formats: &[],
        });

        let texure_view = window_texture.create_view(&TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let color_map = device.create_texture(&TextureDescriptor {
            size: Self::COLOR_MAP_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D1,
            format: TextureFormat::Rgb10a2Unorm,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            label: Some("color map"),
            view_formats: &[],
        });
        let color_view = color_map.create_view(&TextureViewDescriptor::default());

        let color_map_sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor {
            label: Some("color map sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: false },
                            view_dimension: TextureViewDimension::D1,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texure_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&color_view),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::Sampler(&color_map_sampler),
                },
            ],
        });

        const SHADER_DESCRIPTOR: ShaderModuleDescriptor = include_wgsl!("shader.wgsl");
        let shader = device.create_shader_module(SHADER_DESCRIPTOR);

        // print shader compilation errors
        #[cfg(debug_assertions)]
        {
            let compilation_info = shader.get_compilation_info().await;
            println!(
                "{} Shader Compilation Messages",
                compilation_info.messages.len()
            );
            for message in compilation_info.messages {
                println!("{message:?}");
            }
        }

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                // only one entrypoint in the shader
                entry_point: None,
                buffers: &[],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                // only one entry point in the shader
                entry_point: None,
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::COLOR,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),

            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_bind_group,
            streaming_texture: window_texture,
            color_map,
        }
    }

    /// on next render the new palette will be used
    pub fn queue_palette_update(&mut self, palette: Palette<RGB10A2>) {
        let palette = palette.0.map(|c| c.0.to_le_bytes());

        // the queue will remember this and on the next render submission this will be submitted as well
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.color_map,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            palette.as_flattened(),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some((PALETTE_SIZE * size_of::<u32>()) as u32),
                rows_per_image: Some(1),
            },
            Self::COLOR_MAP_SIZE,
        );
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn reinit_surface(&self) {
        if self.size.width > 0 && self.size.height > 0 {
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(
        &mut self,
        framebuffer: &[[u8; WINDOW_SIZE.0]; WINDOW_SIZE.1],
    ) -> Result<(), SurfaceError> {
        // SAFETY:

        let framebuffer = framebuffer.as_flattened();
        assert!(framebuffer.len() == WINDOW_SIZE.0 * WINDOW_SIZE.1);

        const BYTES_PER_ROW: Option<u32> = Some(WINDOW_SIZE.0 as u32);
        const ROWS_PER_IMAGE: Option<u32> = Some(WINDOW_SIZE.1 as u32);

        // push framebuffer to GPU-Texture
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.streaming_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            framebuffer,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: BYTES_PER_ROW,
                rows_per_image: ROWS_PER_IMAGE,
            },
            Self::TEXTURE_SIZE,
        );

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

        render_pass.draw(0..3, 0..1);
        // before finishing the encoding the render_pass must be dropped
        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
}
