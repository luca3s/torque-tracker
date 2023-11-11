use font8x8::UnicodeFonts;
use wgpu::{
    AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendState,
    ColorTargetState, ColorWrites, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d,
    Face, Features, FilterMode, FragmentState, FrontFace, ImageCopyTexture, ImageDataLayout,
    Instance, InstanceDescriptor, Limits, MultisampleState, Operations, Origin3d,
    PipelineLayoutDescriptor, PolygonMode, PowerPreference, PrimitiveState, PrimitiveTopology,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceConfiguration,
    SurfaceError, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexState,
};

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use self::palettes::{Color, Palette, CAMOUFLAGE};

// use self::palettes::Color;

mod palettes;

pub fn draw_single_char(
    character: [u8; 8],
    position: (usize, usize),
    foreground: Color,
    background: Color,
    mut screen: &mut [u8],
) {
    for (y, line) in character.iter().enumerate() {
        for x in 0..8 {
            let color = match (line >> x) & 1 == 1 {
                true => foreground,
                false => background,
            };
            screen[4 * ((y * WINDOW_SIZE.0) + x) + 0] = color[0];
            screen[4 * ((y * WINDOW_SIZE.0) + x) + 1] = color[1];
            screen[4 * ((y * WINDOW_SIZE.0) + x) + 2] = color[2];
            // write alpha value to zero, as it doesnt change ever
            screen[4 * ((y * WINDOW_SIZE.0) + x) + 3] = 0;
        }
    }
}

const WINDOW_TITLE: &str = "RustRacker";
const FONT_SIZE: usize = 8;
const WINDOW_SIZE: (usize, usize) = (640, 400); // (FONT_SIZE * 80, FONT_SIZE * 50)
const PIXEL_SIZE: usize = 4;
const LINE_SIZE: usize = 2560; // PIXEL_SIZE * WINDOW_SIZE.0

#[inline]
const fn char_into_screen_pos(position: (u8, u8)) -> (usize, usize) {
    (position.0 as usize * FONT_SIZE, position.1 as usize * FONT_SIZE)
}

#[inline]
const fn screen_into_byte_pos(position: (usize, usize)) -> usize {
    (position.1 * LINE_SIZE) + (position.0 * PIXEL_SIZE)
}

struct RenderState {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: RenderPipeline,

    streaming_texture: Texture,
    texture_size: Extent3d,

    window: Window,

    diffuse_bind_group: BindGroup,

    framebuffer: [u8; 1_024_000], // WINDOW_SIZE.0 * WINDOW_SIZE.1 * PIXEL_SIZE
    color_palette: Palette,
}

impl RenderState {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let softbuffer: [u8; WINDOW_SIZE.0 * WINDOW_SIZE.1 * 4] =
            [0; WINDOW_SIZE.0 * WINDOW_SIZE.1 * 4];

        let texture_size = Extent3d {
            width: WINDOW_SIZE.0 as u32,
            height: WINDOW_SIZE.1 as u32,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("Streaming Texture"),
            view_formats: &[],
        });

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &softbuffer,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * WINDOW_SIZE.0 as u32),
                rows_per_image: Some(WINDOW_SIZE.1 as u32),
            },
            texture_size,
        );

        let texure_view = texture.create_view(&TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
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
            ],
            label: Some("diffuse_bind_group"),
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Shader"),
            source: ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

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
                entry_point: "vs_main", // 1.
                buffers: &[],           // 2.
            },
            fragment: Some(FragmentState {
                // 3.
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    // 4.
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
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
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_bind_group,
            framebuffer: softbuffer,
            streaming_texture: texture,
            texture_size,
            color_palette: CAMOUFLAGE,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        self.render_texture();

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
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
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);

            render_pass.draw(0..6, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn render_texture(&mut self) {
        // self.framebuffer
        //     .chunks_exact_mut(4)
        //     .for_each(|pixel| pixel.copy_from_slice(&[255, 0, 0, 0]));
        // self.draw_single_char(font8x8::BASIC_FONTS.get('a').unwrap(), (10, 10), 0, 1);

        self.draw_constant();
        // self.draw_string("test 123", (0, 0), 0, 2);

        // push framebuffer onto GPU Texture
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &self.streaming_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &self.framebuffer,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * WINDOW_SIZE.0 as u32),
                rows_per_image: Some(WINDOW_SIZE.1 as u32),
            },
            self.texture_size,
        );
    }

    fn draw_string(
        &mut self,
        string: &str,
        position: (u8, u8),
        fg_color: usize,
        bg_color: usize,
    ) {
        for (num, char) in string.char_indices() {
            self.draw_single_char(
                font8x8::BASIC_FONTS.get(char).unwrap(),
                (position.0 + num as u8, position.1),
                fg_color,
                bg_color,
            );
        }
    }

    fn draw_single_char(
        &mut self,
        char_data: [u8; 8],
        position: (u8, u8),
        fg_color: usize,
        bg_color: usize,
    ) {
        // let top_left_pixel = 4 * ((position.1 * WINDOW_SIZE.0) + position.0);
        let position = char_into_screen_pos(position);
        for (y, line) in char_data.iter().enumerate() {
            for x in 0..8 {
                let color = match (line >> x) & 1 == 1 {
                    true => fg_color,
                    false => bg_color,
                };
                let pixel = 4 * (((position.1 + y) * WINDOW_SIZE.0) + position.0 + x);
                self.framebuffer[pixel] = self.color_palette[color][0];
                self.framebuffer[pixel + 1] = self.color_palette[color][1];
                self.framebuffer[pixel + 2] = self.color_palette[color][2];
                self.framebuffer[pixel + 3] = 0;
            }
        }
    }

    fn draw_rect(&mut self, color: usize, topleft: (u8, u8), botright: (u8, u8)) {
        let color = [
            self.color_palette[color][0],
            self.color_palette[color][1],
            self.color_palette[color][2],
            0,
        ];
        let topleft = char_into_screen_pos(topleft);
        let botright = char_into_screen_pos(botright);
        self.framebuffer
            .chunks_exact_mut(LINE_SIZE)
            .enumerate()
            .filter(|(y, _)| topleft.1 <= *y && *y < botright.1)
            .for_each(|(_, data)| {
                data.chunks_exact_mut(PIXEL_SIZE)
                    .enumerate()
                    .filter(|(x, _)| topleft.0 <= *x && *x < botright.0)
                    .for_each(|(_, pixel)| pixel.copy_from_slice(&color))
            });
    }

    fn draw_constant(&mut self) {
        self.draw_rect(2, (0, 0), (80, 11));
        self.draw_string("Rust Tracker", (34, 1), 0, 2);
        self.draw_string("Song Name", (2, 3), 0, 2);
        self.draw_string("File Name", (2, 4), 0, 2);
        self.draw_string("Order", (6, 5), 0, 2);
        self.draw_string("Pattern", (4, 6), 0, 2);
        self.draw_string("Row", (8, 7), 0, 2);
        self.draw_string("Speed/Tempo", (38, 4), 0, 2);
        self.draw_string("Octave", (43, 5), 0, 2);
        self.draw_string("F1...Help       F9.....Load", (21, 6), 0, 2);
        self.draw_string("ESC..Main Menu  F5/F8..Play / Stop", (21, 7), 0, 2);
        self.draw_string("Time", (63, 9), 0, 2);
        self.draw_string("/", (15, 5), 1, 0);
        self.draw_string("/", (15, 6), 1, 0);
        self.draw_string("/", (15, 7), 1, 0);
        self.draw_string("/", (53, 4), 1, 0);
        self.draw_string("/", (52, 3), 1, 0);

        // except for borders, visual candy can be added later
    }

    fn clear_frame_buffer(&mut self, color: usize) {
        let color = [
            self.color_palette[color][0],
            self.color_palette[color][1],
            self.color_palette[color][2],
            0,
        ];
        self.framebuffer
            .chunks_exact_mut(4)
            .for_each(|pixel| pixel.copy_from_slice(&color));
    }
}

pub async fn run_event_loop() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = RenderState::new(window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id,
            ref event,
        } if window_id == state.window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(pyhsical_size) => {
                state.resize(*pyhsical_size);
            }
            WindowEvent::ScaleFactorChanged {
                new_inner_size,
                scale_factor: _,
            } => {
                state.resize(**new_inner_size);
            }
            _ => {}
        },
        Event::RedrawRequested(window_if) if window_if == state.window().id() => {
            match state.render() {
                Ok(_) => {}
                Err(SurfaceError::Lost) => state.resize(state.size),
                Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprint!("{:?}", e),
            }
        }
        Event::MainEventsCleared => state.window().request_redraw(),
        _ => {}
    });
}
