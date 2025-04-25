use std::sync::Arc;

use eframe::wgpu::{self, util::DeviceExt};

use crate::{Camera, Complex};

use super::fractal::Fractal;

// TODO: change depth to be a u16
const CYCLE_DEPTH: u32 = u32::MAX;

#[derive(Clone, Copy, Debug)]
pub(crate) enum MandelbrotColoring {
    LogDepth,
    HighlightCycle,
}

pub(crate) struct Mandelbrot {
    // position and size
    // TODO: maybe store a Params and not these
    camera: Camera,
    width: u32,
    height: u32,

    // internal stuff
    texture_id: eframe::egui::TextureId,
    needs_update: bool,

    // mandelbrot specific stuff
    z0: Complex,
    max_depth: u32,

    // render pipeline
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Arc<eframe::egui::mutex::RwLock<eframe::egui_wgpu::Renderer>>,
    texture: wgpu::Texture,
    shader_params_buffer: wgpu::Buffer,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}
impl Mandelbrot {
    pub(crate) fn default(cc: &eframe::CreationContext<'_>) -> Self {
        Self::new(
            cc,
            Camera::default(),
            // this needs to be nonzero to make it happy
            100,
            100,
            Complex::zero(),
            // 8192,
            1024,
            MandelbrotColoring::LogDepth,
        )
    }

    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        camera: Camera,
        width: u32,
        height: u32,
        z0: Complex,
        max_depth: u32,
        coloring: MandelbrotColoring,
    ) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let device = render_state.device.clone();
        let queue = render_state.queue.clone();
        let renderer = render_state.renderer.clone();

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("mandelbrot.wgsl"));

        let shader_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shader_params_buffer"),
            contents: bytemuck::bytes_of(&Params::new(camera, width, height, z0, max_depth)),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST,
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        let texture_id = renderer.write().register_native_texture(
            &device,
            &texture.create_view(&wgpu::TextureViewDescriptor::default()),
            wgpu::FilterMode::Nearest,
        );

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("render_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(
                            wgpu::BufferSize::new(shader_params_buffer.size()).unwrap(),
                        ),
                    },
                    count: None,
                }],
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_bind_group"),
            layout: &render_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: shader_params_buffer.as_entire_binding(),
            }],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render_pipeline_layout"),
                    bind_group_layouts: &[&render_bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[
                    // // @location(0) vertex_pos: vec2<f32>,
                    // wgpu::VertexBufferLayout {
                    //     array_stride: 4 * 2,
                    //     step_mode: wgpu::VertexStepMode::Vertex,
                    //     attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    // },
                    // // @location(1) particle_pos: vec2<f32>,
                    // wgpu::VertexBufferLayout {
                    //     array_stride: 4 * 2,
                    //     step_mode: wgpu::VertexStepMode::Instance,
                    //     attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                    // },
                    // // @location(2) particle_vel: vec2<f32>,
                    // wgpu::VertexBufferLayout {
                    //     array_stride: 4 * 2,
                    //     step_mode: wgpu::VertexStepMode::Instance,
                    //     attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                    // },
                    // // @location(3) particle_species: u32,
                    // wgpu::VertexBufferLayout {
                    //     array_stride: 4,
                    //     step_mode: wgpu::VertexStepMode::Instance,
                    //     attributes: &wgpu::vertex_attr_array![3 => Uint32],
                    // },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                // targets: &[Some(config.view_formats[0].into())],
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture.format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            camera,
            width,
            height,
            texture_id,
            needs_update: true,
            z0,
            max_depth,
            device,
            queue,
            renderer,
            texture,
            shader_params_buffer,
            render_bind_group,
            render_pipeline,
        }
    }
}
impl Fractal for Mandelbrot {
    fn needs_update(&self) -> bool {
        self.needs_update
    }

    fn texture_id(&self) -> eframe::egui::TextureId {
        self.texture_id
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn set_width(&mut self, width: u32) {
        if self.width != width {
            self.width = width;
            self.needs_update = true;
        }
    }

    fn set_height(&mut self, height: u32) {
        if self.height != height {
            self.height = height;
            self.needs_update = true;
        }
    }

    fn pan(&mut self, pan: eframe::egui::Vec2) {
        // fn pan_zoom(&mut self, multi_touch: eframe::egui::MultiTouchInfo) {
        // self.camera.pan_zoom(pan, zoom);
        // let pan = multi_touch.translation_delta;
        if pan.x == 0.0 && pan.y == 0.0 {
            return;
        }

        self.camera.center.real -= 2.0 * pan.x / self.width as f32 * self.camera.radius_real;
        self.camera.center.imag += 2.0 * pan.y / self.height as f32
            * (self.camera.radius_real * self.height as f32 / self.width as f32);

        self.needs_update = true;
    }

    fn zoom(&mut self, mouse: eframe::egui::Vec2, zoom: f32) {
        if zoom == 1.0 {
            return;
        }

        self.pan(-mouse);
        self.camera.radius_real /= zoom;
        self.pan(mouse);
        self.needs_update = true;
    }

    fn render_to_texture(&mut self) {
        self.needs_update = false;

        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("command_encoder"),
                });

        self.queue.write_buffer(
            &self.shader_params_buffer,
            0,
            bytemuck::bytes_of(&Params::new(
                self.camera,
                self.width,
                self.height,
                self.z0,
                self.max_depth,
            )),
        );

        // render pass
        command_encoder.push_debug_group("render_pass");
        {
            let new_size = wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            };
            if self.texture.size() != new_size {
                // println!("self.texture.size() != new_size");
                self.texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("texture"),
                    size: wgpu::Extent3d {
                        width: self.width,
                        height: self.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
                });
                self.renderer.write().update_egui_texture_from_wgpu_texture(
                    &self.device,
                    &self
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    eframe::wgpu::FilterMode::Nearest,
                    self.texture_id,
                );
            }
            let texture_view = self
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations::default(),
            })];
            // self.queue.write_buffer(
            //     &self.vertex_buffer,
            //     0,
            //     bytemuck::bytes_of(&get_triangle(
            //         view_settings.particle_radius * view_settings.zoom_scale,
            //     )),
            // );
            let render_pass_descriptor = wgpu::RenderPassDescriptor {
                label: Some("render_pass_descriptor"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.draw(0..3, 0..sim_settings.particle_n as _);
            render_pass.draw(0..6, 0..1);
        }
        command_encoder.pop_debug_group();

        self.queue.submit([command_encoder.finish()]);
        // dbg!(cpu_readable_buffer);
    }
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct Params {
    // shared for all fractals
    // lo_real: f32,
    // lo_imag: f32,
    // hi_real: f32,
    // hi_imag: f32,
    center_real: f32,
    center_imag: f32,
    radius_real: f32,
    radius_imag: f32,
    width: u32,
    height: u32,

    // mandelbrot specific stuff
    z0_real: f32,
    z0_imag: f32,
    max_depth: u32,
    cycle_depth: u32, // this is a ~sentinel for if it finds a cycle
}
impl Params {
    fn new(camera: Camera, width: u32, height: u32, z0: Complex, max_depth: u32) -> Self {
        Params {
            center_real: camera.center.real,
            center_imag: camera.center.imag,
            radius_real: camera.radius_real,
            radius_imag: camera.radius_real * height as f32 / width as f32,
            width,
            height,
            z0_real: z0.real,
            z0_imag: z0.imag,
            max_depth,
            cycle_depth: CYCLE_DEPTH,
        }
    }
}

// fn create_texture(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::Texture {
//     device.create_texture(&wgpu::TextureDescriptor {
//         label: Some("texture"),
//         size,
//         mip_level_count: 1,
//         sample_count: 1,
//         dimension: wgpu::TextureDimension::D2,
//         format: wgpu::TextureFormat::Rgba8UnormSrgb,
//         usage: wgpu::TextureUsages::TEXTURE_BINDING
//             | wgpu::TextureUsages::RENDER_ATTACHMENT
//             | wgpu::TextureUsages::COPY_DST,
//         view_formats: &[eframe::wgpu::TextureFormat::Rgba8UnormSrgb],
//     })
// }
