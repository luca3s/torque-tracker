use std::mem;

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
    event::{Event, Modifiers, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use self::{
    palettes::{Color, Palette},
    ui::{header::Header, pages::pages::{AllPages, Page}},
};

mod ui;

mod palettes;

#[derive(Clone, Copy)]
pub struct CharRect {
    pub top: u8,
    pub bot: u8,
    pub right: u8,
    pub left: u8,
}

impl CharRect {
    pub fn new(top: u8, bot: u8, right: u8, left: u8) -> Self {
        assert!(top <= bot, "top needs to be above bot");
        assert!(left <= right, "left needs to be smaller than right");
        Self { top, bot, right, left }
    }
}

pub struct ScreenRect {
    top: usize,
    bot: usize,
    right: usize,
    left: usize,
}

impl From<CharRect> for ScreenRect {
    fn from(value: CharRect) -> Self {
        Self {
            top: char_into_screen_pos(value.top),
            bot: char_into_screen_pos(value.bot),
            right: char_into_screen_pos(value.right),
            left: char_into_screen_pos(value.left),
        }
    }
}

const WINDOW_TITLE: &str = "RustRacker";
const FONT_SIZE: usize = 8;
const WINDOW_SIZE: (usize, usize) = (FONT_SIZE * 80, FONT_SIZE * 50);
const PIXEL_SIZE: usize = 4;
const LINE_SIZE: usize = PIXEL_SIZE * WINDOW_SIZE.0;

#[inline]
const fn char_into_screen_pos(position: u8) -> usize {
    position as usize * FONT_SIZE
}

#[inline]
const fn screen_into_byte_pos(position: (usize, usize)) -> usize {
    (position.1 * LINE_SIZE) + (position.0 * PIXEL_SIZE)
}

pub(crate) struct DrawBuffer {
    framebuffer: [u8; WINDOW_SIZE.0 * WINDOW_SIZE.1 * PIXEL_SIZE],
    color_palette: Palette,
}

impl DrawBuffer {
    fn new() -> Self {
        Self {
            framebuffer: [0; WINDOW_SIZE.0 * WINDOW_SIZE.1 * 4],
            color_palette: palettes::CAMOUFLAGE,
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
        let position = (
            char_into_screen_pos(position.0),
            char_into_screen_pos(position.1),
        );
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

    fn draw_box(&mut self, rect: CharRect, color_inverse: bool) {
        let outer_color = 0;
        let (inner_top_left_color, inner_bot_right_color) = match color_inverse {
            true => (1, 3),
            false => (3, 1),
        };

        // top left corner
        self.draw_single_char(font8x8::BLOCK_UNICODE[23].into(), (rect.left, rect.top), inner_top_left_color, outer_color);
        // top right corner
        self.draw_single_char(font8x8::BLOCK_UNICODE[22].into(), (rect.right, rect.top), inner_bot_right_color, outer_color);
        // bot left corner
        self.draw_single_char(font8x8::BLOCK_UNICODE[29].into(), (rect.left, rect.bot), inner_top_left_color, outer_color);
        // bot right corner
        self.draw_single_char(font8x8::BLOCK_UNICODE[24].into(), (rect.right, rect.bot), inner_bot_right_color, outer_color);

        // bot & top border
        for i in rect.left+1..rect.right {
            println!("bot, top: {:?}", i);
            self.draw_single_char(font8x8::BLOCK_UNICODE[4].into(), (i, rect.top), inner_top_left_color, outer_color);
            self.draw_single_char(font8x8::BLOCK_UNICODE[0].into(), (i, rect.bot), inner_bot_right_color, outer_color);
        }

        // left & right border
        for i in rect.top+1..rect.bot {
            println!("left, right: {:?}", i);
            self.draw_single_char(font8x8::BLOCK_UNICODE[12].into(), (rect.left, i), outer_color, inner_top_left_color);
            self.draw_single_char(font8x8::BLOCK_UNICODE[16].into(), (rect.right, i), outer_color, inner_bot_right_color);
        }
    }

    fn draw_rect(&mut self, color: usize, rect: CharRect) {
        let color = [
            self.color_palette[color][0],
            self.color_palette[color][1],
            self.color_palette[color][2],
            0,
        ];
        let screen_pos = ScreenRect::from(rect);
        // let topleft = char_into_screen_pos(topleft);
        // let botright = char_into_screen_pos(botright);
        self.framebuffer
            .chunks_exact_mut(LINE_SIZE)
            .enumerate()
            .filter(|(y, _)| screen_pos.top <= *y && *y < screen_pos.bot)
            .for_each(|(_, data)| {
                data.chunks_exact_mut(PIXEL_SIZE)
                    .enumerate()
                    .filter(|(x, _)| screen_pos.left <= *x && *x < screen_pos.right)
                    .for_each(|(_, pixel)| pixel.copy_from_slice(&color))
            });
    }

    fn draw_string(&mut self, string: &str, position: (u8, u8), fg_color: usize, bg_color: usize) {
        for (num, char) in string.char_indices() {
            self.draw_single_char(
                font8x8::BASIC_FONTS.get(char).unwrap(),
                (position.0 + num as u8, position.1),
                fg_color,
                bg_color,
            );
        }
    }

    fn clear(&mut self, color: usize) {
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

pub(crate) struct WindowState {
    surface: Surface<'static>,

    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: RenderPipeline,

    streaming_texture: Texture,
    texture_size: Extent3d,

    diffuse_bind_group: BindGroup,
    // draw_buffer: Box<DrawBuffer>,
}

impl WindowState {
    async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = Instance::default();

        let surface = instance.create_surface(window).unwrap();

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
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
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
            desired_maximum_frame_latency: 2, // default
        };
        surface.configure(&device, &config);

        // let softbuffer: [u8; WINDOW_SIZE.0 * WINDOW_SIZE.1 * 4] =
        //     [0; WINDOW_SIZE.0 * WINDOW_SIZE.1 * 4];

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
            &[0; WINDOW_SIZE.0 * WINDOW_SIZE.1 * 4],
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
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_bind_group,
            streaming_texture: texture,
            texture_size,
            // draw_buffer: Box::new(DrawBuffer::new())
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self, framebuffer: &[u8]) -> Result<(), SurfaceError> {
        // self.render_texture();

        // push framebuffer to GPU-Texture
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &self.streaming_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            framebuffer,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * WINDOW_SIZE.0 as u32),
                rows_per_image: Some(WINDOW_SIZE.1 as u32),
            },
            self.texture_size,
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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
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
}

pub async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut window_state = WindowState::new(window).await;
    let mut draw_buffer = DrawBuffer::new();
    let mut modifiers = Modifiers::default();
    let mut pages = AllPages::new();

    let ui_header = Header {};
    ui_header.draw_constant(&mut draw_buffer);

    let _ = event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            window_id: _, // can ignore because i only use one window
            ref event,
        } => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(pyhsical_size) => {
                window_state.resize(*pyhsical_size);
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _,
            } => {
                // window_state.resize(**new_inner_size);
                // due to a version bump in winit i dont know anymore how to handle this event so i just ignore it for know and see if it makes problems in the future
                println!("Window Scale Factor Changed");
            }
            WindowEvent::RedrawRequested => {
                // draw the new frame buffer
                pages.draw(&mut draw_buffer);

                // push the framebuffer into GPU and render it onto the screen
                match window_state.render(&draw_buffer.framebuffer) {
                    Ok(_) => {}
                    Err(SurfaceError::Lost) => window_state.resize(window_state.size),
                    Err(SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprint!("{:?}", e),
                }
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => if !is_synthetic {},
            // not sure if i need it just to make sure i always have all current modifiers to be used with keyboard events
            WindowEvent::ModifiersChanged(new_modifiers) => modifiers = *new_modifiers,
            _ => {}
        },

        // Event::RedrawRequested(window_id) if window_id == window_state.window().id() => {
        //     // draw the new frame buffer

        //     // push the framebuffer into GPU and render it onto the screen
        //     match window_state.render(&draw_buffer.framebuffer) {
        //         Ok(_) => {}
        //         Err(SurfaceError::Lost) => window_state.resize(window_state.size),
        //         Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
        //         Err(e) => eprint!("{:?}", e),
        //     }
        // }
        // Event::MainEventsCleared => window_state.window().request_redraw(),
        Event::UserEvent(()) => (),
        _ => {}
    });
}
