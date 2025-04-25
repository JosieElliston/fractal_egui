mod fractals;

use eframe::egui::{self, Vec2};
use fractals::{fractal::*, mandelbrot::Mandelbrot};

fn main() -> eframe::Result {
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "fractal",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Clone, Copy, Debug)]
struct Complex {
    real: f32,
    imag: f32,
}
impl Complex {
    fn zero() -> Self {
        Self {
            real: 0.0,
            imag: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
// struct Camera {
//     lo: Complex,
//     hi: Complex,
// }
struct Camera {
    center: Complex,
    radius_real: f32,
}
impl Camera {
    // fn from_center_radius(center: Complex, radius: f32) -> Self {
    //     Self {
    //         lo: Complex {
    //             real: center.real - radius,
    //             imag: center.imag - radius,
    //         },
    //         hi: Complex {
    //             real: center.real + radius,
    //             imag: center.imag + radius,
    //         },
    //     }
    // }

    // fn center(&self) -> Complex {
    //     Complex {
    //         real: (self.lo.real + self.hi.real) / 2.0,
    //         imag: (self.lo.imag + self.hi.imag) / 2.0,
    //     }
    // }
}
impl Default for Camera {
    fn default() -> Self {
        // Self {
        //     lo: Complex {
        //         real: -2.0,
        //         imag: -2.0,
        //     },
        //     hi: Complex {
        //         real: 2.0,
        //         imag: 2.0,
        //     },
        // }
        Self {
            center: Complex {
                real: 0.0,
                imag: 0.0,
            },
            radius_real: 2.0,
        }
    }
}

// struct Fractal {
//     fractal: Box<dyn Fractal>,
//     needs_update: bool,
// }
// impl Fractal {
//     // fn new(fractal: impl Fractal) -> Self {
//     //     Self {
//     //         fractal: Box::new(fractal),
//     //         needs_update: true,
//     //         camera: Camera::default(),
//     //     }
//     // }

//     fn render(&mut self, ui: &eframe::egui::Ui) {
//         if self.needs_update {
//             self.fractal.render_to_texture();
//             self.needs_update = false;
//         }
//         eframe::egui::widgets::Image::from_texture(eframe::egui::load::SizedTexture::new(
//             self.fractal.texture_id,
//             eframe::egui::Vec2::new(10.0, 10.0), // arbitrary size
//         ))
//         .paint_at(ui, ui.available_rect_before_wrap());
//     }
// }

struct FractalWindow {
    fractal: Box<dyn Fractal>,
    is_open: bool,
}

struct App {
    main: Box<dyn Fractal>,
    windows: Vec<FractalWindow>,
    show_overlay: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            main: Box::new(Mandelbrot::default(cc)),
            windows: vec![],
            show_overlay: true,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(egui::Frame::new())
            .show(ctx, |ui| {
                let dt = ctx.input(|input_state| input_state.stable_dt);
                // println!("dt: {:?}", dt);
                // let _ = ui.button(format!("dt: {:?}", dt));
                // let _ = ui.button(format!("1/dt: {:?}", 1.0 / dt));

                // let scale = ui.available_rect_before_wrap().size().min_elem();
                // let rect = egui::Rect::from_min_size(
                //     ui.available_rect_before_wrap().min,
                //     Vec2::new(scale, scale),
                // );

                // self.view_settings.texture_size = 2 * scale as u32;
                // 2 * because something (maybe at the os level?) does antialiasing better with that
                // let mut unit = scale / (2. * self.view_settings.zoom_scale); // TODO: what is this

                // pan and zoom
                let r = ui.interact(
                    ui.available_rect_before_wrap(),
                    eframe::egui::Id::new("main"),
                    egui::Sense::click_and_drag(),
                );
                self.main.pan(ctx.input(|i| i.smooth_scroll_delta));
                self.main.pan(r.drag_delta());

                // TODO: zooming with mouse

                // egui::Sense::interactive(&self.view_settings)
                // r.sense.set(egui::Sense::drag());

                // println!("{}", ctx.input(|i| i.zoom_delta()));
                if let Some(mouse_pos) = ctx.input(|i| i.pointer.latest_pos()) {
                    self.main.zoom(
                        mouse_pos - ui.available_rect_before_wrap().center(),
                        ctx.input(|i| i.zoom_delta()),
                    )
                    // self.main.zoom(
                    //     mouse_pos - ui.available_rect_before_wrap().center(),
                    //     ctx.input(|i| i.smooth_scroll_delta.y),
                    // );
                    // ctx.input(|i| {
                    //     i.events.iter().for_each(|e| {
                    //         if let egui::Event::MouseWheel {
                    //             unit,
                    //             delta,
                    //             modifiers,
                    //         } = e
                    //         {
                    //             let zoom = match unit {
                    //                 egui::MouseWheelUnit::Point => delta.y / 10000.0,
                    //                 egui::MouseWheelUnit::Line => todo!(),
                    //                 egui::MouseWheelUnit::Page => todo!(),
                    //             };
                    //             self.main.zoom(
                    //                 mouse_pos - ui.available_rect_before_wrap().center(),
                    //                 zoom,
                    //             )
                    //         }
                    //     })
                    // });
                }

                // if r.hovered() {
                //     let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y / unit);
                //     if scroll_delta.abs() > 0.001 {
                //         self.view_settings.zoom_scale = (self.view_settings.zoom_scale - scroll_delta).max(0.1);
                //         unit = scale / (2. * self.view_settings.zoom_scale);
                //     }
                // }

                // let scale = egui_rect.size() / (1. * egui_rect.size().min_elem());
                // let scale = [scale.x * self.scale, scale.y * self.scale];

                // let screen_to_egui =
                //     |pos: Pos| pos2(pos.x as f32, -pos.y as f32) * unit + cen.to_vec2();
                // let egui_to_screen = |pos: Pos2| {
                //     let pos = (pos - cen.to_vec2()) / unit;
                //     Pos {
                //         x: pos.x as f64,
                //         y: -pos.y as f64,
                //     }
                // };

                // if r.dragged_by(egui::PointerButton::Secondary) && r.drag_delta().length() > 0.1 {
                //     if let Some(mpos) = r.interact_pointer_pos() {
                //         let egui_to_geom = |pos: Pos2| {
                //             let Pos { x, y } = egui_to_screen(pos);
                //             cga2d::point(x, y)
                //         };
                //         let root_pos = egui_to_geom(mpos - r.drag_delta());
                //         let end_pos = egui_to_geom(mpos);

                //         let modifiers = ctx.input(|i| i.modifiers);

                //         let ms: Vec<cga2d::Blade3> = self
                //             .tiling
                //             .mirrors
                //             .iter()
                //             .map(|&m| self.camera_transform.sandwich(m))
                //             .collect();
                //         let boundary = match (modifiers.command, modifiers.alt) {
                //             (true, false) => {
                //                 let third = if self.tiling.rank == 4 {
                //                     !ms[3]
                //                 } else {
                //                     !(!ms[0] ^ !ms[1] ^ !ms[2])
                //                 };
                //                 !ms[1] ^ !ms[2] ^ third
                //             }
                //             (false, true) => {
                //                 let third = if self.tiling.rank == 4 {
                //                     !ms[3]
                //                 } else {
                //                     !(!ms[0] ^ !ms[1] ^ !ms[2])
                //                 };
                //                 !ms[0] ^ !ms[1] ^ third
                //             }
                //             _ => !ms[0] ^ !ms[1] ^ !ms[2],
                //         }; // the boundary to fix when transforming space

                //         let init_refl = !(root_pos ^ end_pos) ^ !boundary; // get root_pos to end_pos
                //         let f = end_pos ^ !boundary;
                //         let final_refl = !(!init_refl ^ f) ^ f; // restore orientation fixing the "straight line" from root_pos to end_pos

                //         self.camera_transform =
                //             (final_refl * init_refl * self.camera_transform).normalize();
                //     }
                // }

                // egui::Window::new("My Window").show(ctx, |ui| {
                //     ui.label("Hello World!");
                // });

                if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                    self.show_overlay = !self.show_overlay;
                }

                self.main.render_to_ui(ui);

                if self.show_overlay {
                    for window in &mut self.windows {
                        // TODO: better title
                        // TODO: make title smaller
                        egui::Window::new("Fractal")
                            // .open(&mut is_open)
                            // .vscroll(false)
                            .resizable(true)
                            // .title_bar(false)
                            // .default_open(default_open)
                            .default_size([250.0, 250.0])
                            .open(&mut window.is_open)
                            .resizable(true)
                            .show(ctx, |ui| {
                                window.fractal.render_to_ui(ui);
                                ui.allocate_space(ui.available_size());
                            });
                    }

                    // settings ui
                    egui::Frame::popup(ui.style())
                        // .outer_margin(10.0)
                        // .shadow(egui::Shadow::NONE)
                        // .stroke(egui::Stroke::NONE)
                        .show(ui, |ui| {
                            egui::CollapsingHeader::new("settings").show(ui, |ui| {});
                        });
                }
            });
    }
}
