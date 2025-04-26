use std::sync::Arc;

use eframe::{
    egui,
    wgpu::{self, util::DeviceExt},
};

use crate::{Camera, Complex};

const CYCLE_DEPTH: u32 = u32::MAX;
const VELOCITY_DAMPING: f32 = 0.9999;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum FractalType {
    Mandelbrot { z0: Complex } = 0,
    Julia = 1,
}
impl FractalType {
    pub(crate) fn new_mandelbrot(z0: Complex) -> Self {
        Self::Mandelbrot { z0 }
    }
}
// impl Default for FractalType {
//     fn default() -> Self {
//         Self::Mandelbrot { z0: Complex::ZERO }
//     }
// }

#[repr(C)]
#[derive(Clone, Copy, bytemuck::NoUninit)]
struct Params {
    center_real: f32,
    center_imag: f32,
    radius_real: f32,
    radius_imag: f32,
    width: u32,
    height: u32,
    max_depth: u32,
    cycle_depth: u32, // this is a ~sentinel for if it finds a cycle
    escape_radius_2: f32,
    fractal_type: u32,
    point_real: f32,
    point_imag: f32,
}
impl Params {
    fn new(
        camera: Camera,
        width: u32,
        height: u32,

        ty: FractalType,
        max_depth: u32,
        escape_radius: f32,
    ) -> Self {
        // TODO: do this in a better way
        match ty {
            FractalType::Mandelbrot { z0 } => Self {
                center_real: camera.center.real,
                center_imag: camera.center.imag,
                radius_real: camera.radius_real,
                radius_imag: camera.radius_real * height as f32 / width as f32,
                width,
                height,
                max_depth,
                cycle_depth: CYCLE_DEPTH, // this is a ~sentinel for if it finds a cycle
                escape_radius_2: escape_radius * escape_radius,
                fractal_type: 0,
                point_real: z0.real,
                point_imag: z0.imag,
            },
            FractalType::Julia => todo!(),
        }
    }
}

pub(crate) struct FractalUiResponse {
    pub(crate) should_open_settings: bool,
    pub(crate) new_point: Option<Complex>,
}

pub(crate) struct SettingsUiResponse {
    pub(crate) is_settings_open: bool,
    pub(crate) swap_main: bool,
}

pub(crate) struct Fractal {
    // view stuff
    camera: Camera,
    // velocity: Camera,
    // TODO: make this in natural units
    velocity: eframe::egui::Vec2,
    // width: u32,
    // height: u32,
    size: eframe::egui::Vec2,

    // // this is so that we can manually do subsampling,
    // // which allows for reverse subsampling to render at a lower resolution
    // render_width: u32,
    // render_height: u32,
    // output_width: u32,
    // output_height: u32,

    // internal stuff
    texture_id: eframe::egui::TextureId,
    needs_update: bool,

    // render pipeline
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Arc<eframe::egui::mutex::RwLock<eframe::egui_wgpu::Renderer>>,
    texture: wgpu::Texture,
    shader_params_buffer: wgpu::Buffer,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,

    // fractal stuff
    ty: FractalType,
    max_depth: u32,
    escape_radius: f32,
    settings_open: bool,
}
impl Fractal {
    pub(crate) fn default(render_state: &eframe::egui_wgpu::RenderState, ty: FractalType) -> Self {
        Self::new(
            render_state,
            Camera::default(),
            // Camera {
            //     center: Complex::zero(),
            //     radius_real: 0.0,
            // },
            eframe::egui::Vec2::ZERO,
            // these needs to be nonzero to make a texture
            egui::Vec2::new(1.0, 1.0),
            ty,
            1024,
            10.0,
        )
    }

    pub(crate) fn new(
        render_state: &eframe::egui_wgpu::RenderState,
        camera: Camera,
        velocity: eframe::egui::Vec2,
        size: eframe::egui::Vec2,
        ty: FractalType,
        max_depth: u32,
        escape_radius: f32,
    ) -> Self {
        let device = render_state.device.clone();
        let queue = render_state.queue.clone();
        let renderer = render_state.renderer.clone();

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("mandelbrot.wgsl"));

        let shader_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shader_params_buffer"),
            // TODO: i should be able to just zero init this
            // contents: bytemuck::bytes_of(&Params::new(camera, width, height, Co, 199)),
            contents: &[0; std::mem::size_of::<Params>()],
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST,
        });

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("texture"),
            size: wgpu::Extent3d {
                width: size.x as u32,
                height: size.y as u32,
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
            velocity,
            size,
            texture_id,
            needs_update: true,
            device,
            queue,
            renderer,
            texture,
            shader_params_buffer,
            render_bind_group,
            render_pipeline,
            ty,
            max_depth,
            escape_radius,
            settings_open: false,
        }
    }

    pub(crate) fn camera(&self) -> Camera {
        self.camera
    }

    fn texture_id(&self) -> eframe::egui::TextureId {
        self.texture_id
    }

    pub(crate) fn pan(&mut self, pan: eframe::egui::Vec2) {
        // fn pan_zoom(&mut self, multi_touch: eframe::egui::MultiTouchInfo) {
        // self.camera.pan_zoom(pan, zoom);
        // let pan = multi_touch.translation_delta;
        if pan.x == 0.0 && pan.y == 0.0 {
            return;
        }
        self.camera.center.real -= 2.0 * pan.x / self.size.x * self.camera.radius_real;
        self.camera.center.imag += 2.0 * pan.y * (self.camera.radius_real / self.size.x);
        self.needs_update = true;
    }

    pub(crate) fn pan_velocity(&mut self, pan_velocity: eframe::egui::Vec2) {
        self.velocity = if pan_velocity.length_sq() < 0.0001 {
            eframe::egui::Vec2::ZERO
        } else {
            pan_velocity
        };
    }

    pub(crate) fn autopan(&mut self, dt: f32) {
        self.pan(self.velocity * dt);
        self.pan_velocity(self.velocity * (1.0 - VELOCITY_DAMPING).powf(dt));
    }

    pub(crate) fn zoom(&mut self, mouse: eframe::egui::Vec2, zoom: f32) {
        if zoom == 1.0 {
            return;
        }

        self.pan(-mouse);
        self.camera.radius_real /= zoom;
        self.pan(mouse);
        self.needs_update = true;
    }

    pub(crate) fn set_point(&mut self, point: Complex) {
        match &mut self.ty {
            FractalType::Mandelbrot { z0 } => {
                *z0 = point;
                self.needs_update = true;
            }
            FractalType::Julia => todo!(),
        }
    }

    /// fills the entire ui rect with the image.
    /// draws the point if it is Some.
    /// returns whether the settings ui should be open ie we were double clicked.
    pub(crate) fn ui(
        &mut self,
        ctx: &egui::Context,
        ui: &mut eframe::egui::Ui,
        point: Option<Complex>,
    ) -> FractalUiResponse {
        let rect = ui.available_rect_before_wrap();
        let r = ui.allocate_rect(rect, egui::Sense::click_and_drag());

        // camera stuff
        let dt = ctx.input(|input_state| input_state.stable_dt);
        // if self.trackpad {
        //     self.pan(ctx.input(|i| i.smooth_scroll_delta));
        // } else
        // TODO: it's kinda weird that i can't just get is_primary_down_on()
        if r.is_pointer_button_down_on() && ctx.input(|i| i.pointer.primary_down()) {
            self.pan(r.drag_delta());
            self.pan_velocity(r.drag_delta() / dt);
        } else {
            self.autopan(dt);
        }
        // if r.hover_pos()
        if r.contains_pointer() {
            if let Some(mouse_pos) = ctx.input(|i| i.pointer.latest_pos()) {
                self.zoom(
                    mouse_pos - rect.center(),
                    ctx.input(|i| {
                        // if self.trackpad {
                        //     i.zoom_delta()
                        // } else {
                        //     (i.smooth_scroll_delta.y / 300.0).exp()
                        // }
                        (i.smooth_scroll_delta.y / 300.0).exp()
                    }),
                )
            }
        }

        // rendering stuff
        if self.size != rect.size() {
            self.size = rect.size();
            self.needs_update = true;
        }
        self.render_to_texture();
        eframe::egui::widgets::Image::from_texture(eframe::egui::load::SizedTexture::new(
            self.texture_id(),
            eframe::egui::Vec2::new(1.0, 1.0), // arbitrary size
        ))
        .paint_at(ui, rect);
        if let Some(point) = point {
            // TODO: this is probably wrong
            ui.painter_at(rect).circle_filled(
                rect.center()
                    + eframe::egui::Vec2::new(
                        point.real * rect.width() / self.size.x,
                        point.imag * rect.height() / self.size.y,
                    ),
                5.0,
                eframe::egui::Color32::from_black_alpha(200),
            );
        }

        // move the point
        let point = if r.is_pointer_button_down_on()
            && ctx.input(|i: &egui::InputState| i.pointer.secondary_down())
        {
            // TODO: this is probably wrong
            // TODO: this control flow is very bad
            r.hover_pos().map(|p| {
                let x = p.x / rect.width() * self.size.x;
                let y = p.y / rect.height() * self.size.y;
                Complex {
                    real: self.camera.center.real - self.camera.radius_real + x,
                    imag: self.camera.center.imag
                        - self.camera.radius_real * (self.size.y / self.size.x)
                        + y,
                }
            })
        } else {
            None
        };

        FractalUiResponse {
            should_open_settings: r.double_clicked(),
            new_point: point,
        }
    }

    /// returns whether the settings ui should still be open
    pub(crate) fn settings_ui(
        &mut self,
        ctx: &egui::Context,
        ui: &mut eframe::egui::Ui,
        name: &str,
    ) -> SettingsUiResponse {
        let mut open = true;
        let mut swap_main = false;
        egui::Window::new(format!("{name} settings"))
            // .title_bar(false)
            .open(&mut open)
            .show(ctx, |ui| {
                swap_main = ui.button("swap main").clicked();

                ui.label("max depth");
                let mut max_depth = self.max_depth;
                ui.add(
                    egui::Slider::new(&mut max_depth, 1..=1024)
                        .clamping(egui::SliderClamping::Never),
                );
                if max_depth != self.max_depth {
                    self.max_depth = max_depth;
                    self.needs_update = true;
                }

                ui.label("escape radius");
                let mut escape_radius = self.escape_radius;
                ui.add(
                    egui::Slider::new(&mut escape_radius, 0.0..=10.0)
                        .clamping(egui::SliderClamping::Never),
                );
                if escape_radius != self.escape_radius {
                    self.escape_radius = escape_radius;
                    self.needs_update = true;
                }
            });
        SettingsUiResponse { is_settings_open: open, swap_main }
    }

    /// render the fractal to a wgpu texture and resets needs_update
    fn render_to_texture(&mut self) {
        if !self.needs_update {
            return;
        }
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
                self.size.x as u32,
                self.size.y as u32,
                self.ty,
                self.max_depth,
                self.escape_radius,
            )),
        );

        // render pass
        command_encoder.push_debug_group("render_pass");
        {
            let new_size = wgpu::Extent3d {
                width: self.size.x as u32,
                height: self.size.y as u32,
                depth_or_array_layers: 1,
            };
            if self.texture.size() != new_size {
                // println!("self.texture.size() != new_size");
                self.texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("texture"),
                    size: new_size,
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
