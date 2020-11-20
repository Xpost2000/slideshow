extern crate sdl2;

use sdl2::pixels::Color as SDLColor;
use std::collections::HashMap;

use crate::Color;
type SDL2WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;
// hashmaps of hashmaps?
// Yeah this is not a good idea, but whatever for now.

// TODO: convert this to &str.
struct SDL2FontAsset<'ttf> {
    file_name: String,
    stored_sizes : HashMap<u16, sdl2::ttf::Font<'ttf, 'static>>,
}

impl<'ttf> SDL2FontAsset<'ttf> {
    fn new(file_name: String) -> SDL2FontAsset<'ttf> {
        SDL2FontAsset {file_name: file_name.clone(), stored_sizes: HashMap::new(),}
    }

    fn new_with_common_sizes(file_name: String, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) -> SDL2FontAsset<'ttf> {
        SDL2FontAsset {
            file_name: file_name.clone(),
            stored_sizes: {
                let mut okay_table_i_guess = HashMap::new();
                // probably common sizes.
                for font_size in &[10, 11, 12, 14, 16, 18, 24, 32, 36, 48, 64, 72, 84, 96, 128] {
                    okay_table_i_guess.insert(*font_size, ttf_context.load_font(file_name.clone(), *font_size).unwrap());
                }

                okay_table_i_guess
            }
        }
    }

    fn load_size_if_not_loaded(&mut self, font_size: u16, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) {
        if let None = self.stored_sizes.get(&font_size) {
            self.stored_sizes.insert(font_size, ttf_context.load_font(self.file_name.clone(), font_size).unwrap());
        }
    }

    fn get_size(&mut self, font_size: u16, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) -> &sdl2::ttf::Font<'ttf, 'static> {
        self.load_size_if_not_loaded(font_size, ttf_context);
        self.stored_sizes.get(&font_size).unwrap()
    }

    fn get_size_mut(&mut self, font_size: u16, ttf_context: &'ttf sdl2::ttf::Sdl2TtfContext) -> &mut sdl2::ttf::Font<'ttf, 'static> {
        self.load_size_if_not_loaded(font_size, ttf_context);
        self.stored_sizes.get_mut(&font_size).unwrap()
    }
}

pub struct SDL2GraphicsContext<'sdl2, 'ttf> {
    window_canvas : SDL2WindowCanvas,
    ttf_context : &'ttf sdl2::ttf::Sdl2TtfContext,
    font_assets : HashMap<String, SDL2FontAsset<'ttf>>,
    video_subsystem: &'sdl2 sdl2::VideoSubsystem,
}

// lots of interface and safety changes to be made.
impl<'sdl2,'ttf> SDL2GraphicsContext<'sdl2, 'ttf> {
    // this is technically an associated function
    pub fn new(window: sdl2::video::Window,
               ttf_context : &'ttf sdl2::ttf::Sdl2TtfContext,
               video_subsystem: &'sdl2 sdl2::VideoSubsystem) -> SDL2GraphicsContext<'sdl2, 'ttf> {
        SDL2GraphicsContext {
            window_canvas: window.into_canvas().build().unwrap(),
            ttf_context,
            video_subsystem,
            font_assets: HashMap::new()
        }
    }

    pub fn get_avaliable_resolutions(&self) -> Vec<(i32, i32)> {
        let mut resolutions : Vec<(i32, i32)> = Vec::new();
        if let Ok(display_mode_count) = self.video_subsystem.num_display_modes(0) {
            for index in 0..display_mode_count {
                if let Ok(display_mode) = self.video_subsystem.display_mode(0, index) {
                    resolutions.push((display_mode.w, display_mode.h));
                }
            }
        }

        resolutions.dedup();
        resolutions
    }

    pub fn add_image<'a>(&mut self, image_file_name: &'a str) -> &'a str {
        unimplemented!("add_image");
        image_file_name
    }
    pub fn add_font<'a>(&mut self, font_name: &'a str) -> &'a str {
        self.font_assets.insert(font_name.to_owned(),
                                SDL2FontAsset::new_with_common_sizes(font_name.to_owned(), &self.ttf_context));
        font_name
    }

    pub fn toggle_fullscreen(&mut self) {
        use sdl2::video::FullscreenType;
        let fullscreen_state = self.window().fullscreen_state();
        let mut window = self.window_mut();

        window.set_fullscreen(
            match fullscreen_state {
                FullscreenType::Off => {FullscreenType::True},
                FullscreenType::True | FullscreenType::Desktop => {
                    FullscreenType::Off
                },
            }
        );
    }

    pub fn window(&self) -> &sdl2::video::Window {
        self.window_canvas.window()
    }

    pub fn window_mut(&mut self) -> &mut sdl2::video::Window {
        self.window_canvas.window_mut()
    }

    pub fn present(&mut self) {
        self.window_canvas.present();
    }

    pub fn clear_color(&mut self, clear_color: Color) {
        self.window_canvas.set_draw_color(
            SDLColor::RGBA(
                clear_color.r,
                clear_color.g,
                clear_color.b,
                clear_color.a,
            )
        );
        self.window_canvas.clear();
    }

    pub fn get_font_asset(&self, font_id: &str) -> Option<&SDL2FontAsset<'ttf>> {
        self.font_assets.get(font_id)
    } 

    pub fn get_font_asset_mut(&mut self, font_id: &str) -> Option<&mut SDL2FontAsset<'ttf>> {
        self.font_assets.get_mut(font_id)
    } 

    pub fn find_text_asset_by_size(&mut self, font_id: &str, font_size: u16) -> Option<&sdl2::ttf::Font<'ttf, 'static>> {
        let ttf_context = self.ttf_context;
        let font_asset = self.get_font_asset_mut(font_id);

        if let Some(font_asset) = font_asset {
            Some(font_asset.get_size(font_size, ttf_context))
        } else {
            None
        }
    }

    pub fn find_text_asset_by_size_mut(&mut self, font_id: &str, font_size: u16) -> Option<&mut sdl2::ttf::Font<'ttf, 'static>> {
        let ttf_context = self.ttf_context;
        let font_asset = self.get_font_asset_mut(font_id);

        if let Some(font_asset) = font_asset {
            Some(font_asset.get_size_mut(font_size, ttf_context))
        } else {
            None
        }
    }

    pub fn text_dimensions(&mut self, font_id: &str, text: &str, font_size: u16) -> (u32, u32) {
        if let Some(font_at_size) = self.find_text_asset_by_size(font_id, font_size) {
            let (width, height) = font_at_size.size_of(text).unwrap();
            (width, height)
        } else {
            (0, 0)
        }
    }

    pub fn render_image(&mut self, image_id: &str, x: f32, y: f32) {
        unimplemented!("ASIDIOASDIOASDIO");
    }

    pub fn render_text(&mut self, font_id: &str, x: f32, y: f32, text: &str, font_size: u16, color: Color, style: sdl2::ttf::FontStyle) {
        let ttf_context = self.ttf_context;
        let texture_creator = self.window_canvas.texture_creator();

        match self.find_text_asset_by_size_mut(font_id, font_size) {
            Some(font) => {
                if font.get_style() != style {
                    font.set_style(style);
                }
                let mut texture = texture_creator.create_texture_from_surface(
                    &font.render(text)
                        .blended(SDLColor::RGBA(255, 255, 255, 255))
                        .expect("how did this go wrong?")
                ).expect("how did this go wrong?");

                texture.set_color_mod(color.r, color.g, color.b);
                texture.set_alpha_mod(color.a);

                let sdl2::render::TextureQuery { width, height, .. } = texture.query();
                self.window_canvas.copy(&texture, None, Some(sdl2::rect::Rect::new(x as i32, y as i32, width, height))).unwrap();
            },
            None => {}
        }
    }

    pub fn render_filled_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        self.window_canvas.set_draw_color(
            SDLColor::RGBA(
                color.r,
                color.g,
                color.b,
                color.a
            )
        );
        self.window_canvas.fill_rect(sdl2::rect::Rect::new(x as i32, y as i32, w as u32, h as u32));
    }
}
