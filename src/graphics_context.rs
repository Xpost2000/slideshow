extern crate sdl2;

use sdl2::pixels::Color as SDLColor;
use std::collections::HashMap;

use crate::Color;
type SDL2WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;
type SDL2WindowContextTextureCreator = sdl2::render::TextureCreator<sdl2::video::WindowContext>;
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

struct SDL2ImageTextureAsset {
    texture: sdl2::render::Texture,
}

// https://github.com/Rust-SDL2/rust-sdl2/issues/351
// ouch... uh it's a bit too late to fix this part...
// I enabled ["unsafe_textures"] to circumvent this for now...
impl SDL2ImageTextureAsset {
    fn set_color(&mut self, color: Color) {
        self.texture.set_color_mod(color.r, color.g, color.b);
        self.texture.set_alpha_mod(color.a);
    }

    fn dimensions(&self) -> (u32, u32) {
        use sdl2::render::TextureQuery;
        let TextureQuery{ width, height, .. } = self.texture.query(); 
        (width, height)
    }
}

struct SDL2ImageTextureAssets {
    texture_creator : SDL2WindowContextTextureCreator,
    images : HashMap<String, SDL2ImageTextureAsset>,
}

impl SDL2ImageTextureAssets {
    fn new(texture_creator: SDL2WindowContextTextureCreator) -> SDL2ImageTextureAssets {
        SDL2ImageTextureAssets {
            texture_creator,
            images: HashMap::new()
        }
    }

    fn get(&self, id: &str) -> Option<&SDL2ImageTextureAsset> {
        self.images.get(id)
    }

    fn get_mut(&mut self, id: &str) -> Option<&mut SDL2ImageTextureAsset> {
        self.images.get_mut(id)
    }

    fn insert(&mut self, path: &str) {
        use sdl2::image::LoadSurface;
        let surface_image = sdl2::surface::Surface::from_file(path).unwrap();

        self.images.insert(path.to_owned(),
                           SDL2ImageTextureAsset{
                               texture: {
                                   self.texture_creator.create_texture_from_surface(surface_image).unwrap()
                               }
                           });
    }
}

pub struct SDL2GraphicsContext<'sdl2, 'ttf, 'image> {
    window_canvas : SDL2WindowCanvas,
    ttf_context : &'ttf sdl2::ttf::Sdl2TtfContext,
    image_context : &'image sdl2::image::Sdl2ImageContext,
    video_subsystem: &'sdl2 sdl2::VideoSubsystem,

    font_assets : HashMap<String, SDL2FontAsset<'ttf>>,
    image_assets : SDL2ImageTextureAssets,
}

const DEFAULT_DPI : f32 = 96.0;
// lots of interface and safety changes to be made.
impl<'sdl2, 'ttf, 'image> SDL2GraphicsContext<'sdl2, 'ttf, 'image> {
    // this is technically an associated function
    pub fn new(window: sdl2::video::Window,
               ttf_context : &'ttf sdl2::ttf::Sdl2TtfContext,
               image_context : &'image sdl2::image::Sdl2ImageContext,
               video_subsystem: &'sdl2 sdl2::VideoSubsystem) -> SDL2GraphicsContext<'sdl2, 'ttf, 'image> {
        let window_canvas = window.into_canvas().build().unwrap();
        let texture_creator = window_canvas.texture_creator();
        SDL2GraphicsContext {
            window_canvas,
            ttf_context,
            image_context,
            video_subsystem,
            font_assets: HashMap::new(),
            image_assets: SDL2ImageTextureAssets::new(texture_creator),
        }
    }

    fn get_display_dpi(&self) -> (f32, f32, f32) {
        self.video_subsystem.display_dpi(0).unwrap()
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
        self.image_assets.insert(image_file_name);
        image_file_name
    }
    pub fn add_font<'a>(&mut self, font_name: &'a str) -> &'a str {
        self.font_assets.insert(font_name.to_owned(),
                                SDL2FontAsset::new_with_common_sizes(font_name.to_owned(),
                                                                     &self.ttf_context));
        font_name
    }

    pub fn set_resolution(&mut self, resolution_pair: (u32, u32)) {
        use sdl2::video::FullscreenType;
        let fullscreen_state = self.window().fullscreen_state();
        let window = self.window_mut();

        window.set_size(resolution_pair.0, resolution_pair.1);
        match fullscreen_state {
            _ => {}
            FullscreenType::True | FullscreenType::Desktop => {
                let new_display_mode =
                    sdl2::video::DisplayMode{
                        w: resolution_pair.0 as i32,
                        h: resolution_pair.1 as i32,
                        .. window.display_mode().unwrap()
                    };
                println!("{:?}", new_display_mode);
                window.set_display_mode(new_display_mode);
            },
        }
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

    pub fn screen_width(&self) -> u32 {
        self.resolution().0
    }

    pub fn screen_height(&self) -> u32 {
        self.resolution().1
    }

    pub fn resolution(&self) -> (u32, u32) {
        self.window().size()
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

    pub fn get_image_asset(&self, texture_id: &str) -> Option<&SDL2ImageTextureAsset> {
        self.image_assets.get(texture_id)
    }

    pub fn get_image_asset_mut(&mut self, texture_id: &str) -> Option<&mut SDL2ImageTextureAsset> {
        self.image_assets.get_mut(texture_id)
    }

    pub fn get_font_asset(&self, font_id: &str) -> Option<&SDL2FontAsset<'ttf>> {
        self.font_assets.get(font_id)
    } 

    pub fn get_font_asset_mut(&mut self, font_id: &str) -> Option<&mut SDL2FontAsset<'ttf>> {
        self.font_assets.get_mut(font_id)
    } 

    pub fn find_text_asset_by_size(&mut self, font_id: &str, font_size: u16) -> Option<&sdl2::ttf::Font<'ttf, 'static>> {
        let ttf_context = self.ttf_context;
        let font_size : u16 = (font_size as f32 * (self.get_display_dpi().0 / DEFAULT_DPI)) as u16;
        let font_asset = self.get_font_asset_mut(font_id);

        if let Some(font_asset) = font_asset {
            Some(font_asset.get_size(font_size, ttf_context))
        } else {
            None
        }
    }

    pub fn find_text_asset_by_size_mut(&mut self, font_id: &str, font_size: u16) -> Option<&mut sdl2::ttf::Font<'ttf, 'static>> {
        let ttf_context = self.ttf_context;
        let font_size : u16 = (font_size as f32 * (self.get_display_dpi().0 / DEFAULT_DPI)) as u16;
        let font_asset = self.get_font_asset_mut(font_id);

        if let Some(font_asset) = font_asset {
            Some(font_asset.get_size_mut(font_size, ttf_context))
        } else {
            None
        }
    }

    // Please check for whether this image actually exists for real.
    pub fn image_dimensions(&self, texture_image: &str) -> (u32, u32) {
        self.get_image_asset(texture_image).unwrap().dimensions()
    }

    pub fn text_dimensions(&mut self, font_id: &str, text: &str, font_size: u16) -> (u32, u32) {
        if let Some(font_at_size) = self.find_text_asset_by_size(font_id, font_size) {
            let (width, height) = font_at_size.size_of(text).unwrap();
            (width, height)
        } else {
            (0, 0)
        }
    }

    pub fn render_image(&mut self, image_id: &str, x: f32, y: f32, w: f32, h: f32, color: Color) {
        if let Some(texture) = self.get_image_asset_mut(image_id) {
            texture.set_color(color);
        }

        /*
           I'm fairly certain I don't have to do something like this... Should probably
           google this further.
         */
        let &mut SDL2GraphicsContext { ref mut window_canvas, ref image_assets, .. } = self;
        match image_assets.get(image_id) {
            Some(texture) => {
                let texture = &texture.texture;
                window_canvas.copy(texture, None,
                                   Some(sdl2::rect::Rect::new(x as i32,
                                                              y as i32,
                                                              w as u32,
                                                              h as u32)));
            },
            None => {},
        }
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
