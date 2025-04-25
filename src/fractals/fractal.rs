pub(crate) trait Fractal {
    fn needs_update(&self) -> bool;
    fn texture_id(&self) -> eframe::egui::TextureId;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn set_width(&mut self, width: u32);
    fn set_height(&mut self, height: u32);

    fn pan(&mut self, pan: eframe::egui::Vec2);
    fn zoom(&mut self, mouse: eframe::egui::Vec2, zoom: f32);
    // fn pan_zoom(&mut self, multi_touch: eframe::egui::MultiTouchInfo);

    // fn apply_scroll(&mut self, delta: eframe::egui::vec2::Vec2)

    /// render the fractal to a wgpu texture and resets needs_update
    fn render_to_texture(&mut self);

    /// fills the entire ui rect with the image
    fn render_to_ui(&mut self, ui: &eframe::egui::Ui) {
        if self.width() != ui.available_width() as _ {
            self.set_width(ui.available_width() as _);
        }
        if self.height() != ui.available_height() as _ {
            self.set_height(ui.available_height() as _);
        }

        if self.needs_update() {
            // println!("self.needs_update");
            self.render_to_texture();
        } else {
            // println!("cached");
        }

        eframe::egui::widgets::Image::from_texture(eframe::egui::load::SizedTexture::new(
            self.texture_id(),
            eframe::egui::Vec2::new(10.0, 10.0), // arbitrary size
        ))
        .paint_at(ui, ui.available_rect_before_wrap());
    }
}

// #[derive(Clone, Copy, Debug)]
// pub(crate) enum SubSamples {
//     One,
//     Four,
//     Sixteen,
// }
