/*
   For now since I don't want to use a texture atlas to avoid too much work,
   draw_static_text will be a thing.

   It will be the same thing as draw_text, but it will save it to a cache with a hashmap.

TODO: Clear text cache.
*/
extern crate sdl2;

use sdl2::pixels::Color as SDLColor;
use std::collections::HashMap;

use crate::Color;
type SDL2WindowCanvas = sdl2::render::Canvas<sdl2::video::Window>;
type SDL2WindowContextTextureCreator = sdl2::render::TextureCreator<sdl2::video::WindowContext>;
// hashmaps of hashmaps?
// Yeah this is not a good idea, but whatever for now.

// TODO: convert this to &str.
pub struct SDL2FontAsset<'ttf> {
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

pub struct SDL2ImageTextureAsset {
    texture: sdl2::render::Texture,
}

// https://github.com/Rust-SDL2/rust-sdl2/issues/351
// ouch... uh it's a bit too late to fix this part...
// I enabled ["unsafe_textures"] to circumvent this for now...
impl SDL2ImageTextureAsset {
    fn set_blend_mode(&mut self, blend: sdl2::render::BlendMode) {
        self.texture.set_blend_mode(blend);
    }
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

#[derive(Clone,Copy)]
// I would give this associated methods, but it's irrelevant
// without a window.
pub enum VirtualResolution {
    Virtual(u32, u32),
    Display,
}
pub enum TextJustificationHorizontal {
    Left, Right, Center,
}
pub enum TextJustificationVertical {
    Up, Down, Center,
}
pub struct TextJustification(TextJustificationHorizontal, TextJustificationVertical);
impl TextJustification {
    pub fn center() -> TextJustification {
        TextJustification(TextJustificationHorizontal::Center, TextJustificationVertical::Center)
    }
}
// for text justification
pub enum TextBounds {
    EntireScreen,
    Rectangle(f32, f32, f32, f32),
    ScreenLine(f32, f32),
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl Camera {
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0, y: 0.0,
            scale: 1.0,
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct StaticTextCacheKey {
    font_id: String,
    text: String,
    font_size: u16,
    style: sdl2::ttf::FontStyle,
}

pub struct SDL2GraphicsContext<'sdl2, 'ttf, 'image> {
    pub window_canvas : SDL2WindowCanvas,
    ttf_context : &'ttf sdl2::ttf::Sdl2TtfContext,
    image_context : &'image sdl2::image::Sdl2ImageContext,
    video_subsystem: &'sdl2 sdl2::VideoSubsystem,

    white_rectangle_texture: SDL2ImageTextureAsset,
    static_text_texture_cache: HashMap<StaticTextCacheKey, sdl2::render::Texture>,
    font_assets : HashMap<String, SDL2FontAsset<'ttf>>,
    image_assets : SDL2ImageTextureAssets,

    // camera should probably not be public?
    pub camera: Camera,
    pub logical_resolution : VirtualResolution,
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

        let mut white_texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 8, 8).unwrap();
        white_texture.with_lock(None,
                                |buffer: &mut [u8], pitch: usize| {
                                    for y in 0..8 {
                                        for x in 0..8 {
                                            let pixel_start = y * pitch + x * 3;
                                            buffer[pixel_start] = 255;
                                            buffer[pixel_start+1] = 255;
                                            buffer[pixel_start+2] = 255;
                                        }
                                    }
                                })
            .expect("failed to build white pixel texture");

        SDL2GraphicsContext {
            window_canvas,
            ttf_context,
            image_context,
            video_subsystem,
            font_assets: HashMap::new(),
            static_text_texture_cache: HashMap::new(),
            image_assets: SDL2ImageTextureAssets::new(texture_creator),
            white_rectangle_texture: SDL2ImageTextureAsset{ texture: white_texture },
            camera: Camera::default(),
            logical_resolution: VirtualResolution::Display,
        }
    }

    pub fn use_viewport_letterbox(&mut self) {
        let (x,y,w,h) = self.get_letterbox_viewport_rectangle();
        self.window_canvas.set_viewport(
            Some(sdl2::rect::Rect::new(
                x as i32,
                y as i32,
                w as u32,
                h as u32
            )));
    }

    pub fn use_viewport_default(&mut self) {
        self.window_canvas.set_viewport(None);
    }

    pub fn enable_alpha_blending(&mut self) {
        self.window_canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
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
        if !self.font_assets.contains_key(font_name) {
            self.font_assets.insert(font_name.to_owned(),
                                    SDL2FontAsset::new_with_common_sizes(font_name.to_owned(),
                                                                         &self.ttf_context));
        }
        font_name
    }

    pub fn set_resolution(&mut self, resolution_pair: (u32, u32)) {
        use sdl2::video::FullscreenType;
        let fullscreen_state = self.window().fullscreen_state();
        let window = self.window_mut();

        window.set_size(resolution_pair.0, resolution_pair.1)
            .expect("failed to resize window.");

        #[allow(unreachable_patterns)]
        match fullscreen_state {
            FullscreenType::True | FullscreenType::Desktop => {
                let new_display_mode =
                    sdl2::video::DisplayMode{
                        w: resolution_pair.0 as i32,
                        h: resolution_pair.1 as i32,
                        .. window.display_mode().unwrap()
                    };
                window.set_display_mode(new_display_mode)
                    .expect("failed to resize window via display mode.");
            },
            _ => {},
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        use sdl2::video::FullscreenType;
        let fullscreen_state = self.window().fullscreen_state();
        let window = self.window_mut();

        window.set_fullscreen(
            match fullscreen_state {
                FullscreenType::Off => {FullscreenType::True},
                FullscreenType::True | FullscreenType::Desktop => {
                    FullscreenType::Off
                },
            }
        ).expect("failed to change window fullscreen state");
    }

    pub fn logical_width(&self) -> u32 {
        match self.logical_resolution {
            VirtualResolution::Display => {
                self.screen_width()
            },
            VirtualResolution::Virtual(width, _) => {
                width
            }
        }
    }

    pub fn logical_height(&self) -> u32 {
        match self.logical_resolution {
            VirtualResolution::Display => {
                self.screen_height()
            },
            VirtualResolution::Virtual(_, height) => {
                height
            }
        }
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
                                                              h as u32)))
                    .unwrap();
            },
            None => {},
        }
    }

    // also does aspect ratio scaling
    pub fn aspect_ratio(&self) -> f32 {
        self.screen_width() as f32 / self.screen_height() as f32
    }
    pub fn logical_aspect_ratio(&self) -> f32 {
        self.logical_width() as f32 / self.logical_height() as f32
    }
    fn aspect_ratio_scale_factor(&self) -> f32 {
        let scaling_factor = if self.aspect_ratio() > self.logical_aspect_ratio() {
            // tall
            self.screen_height() as f32 / self.logical_height() as f32
        } else {
            // widescreen
            self.screen_width() as f32 / self.logical_width() as f32
        };
        scaling_factor
    }

    fn scale_and_transform_xy_pair_to_real(&self, x: f32, y: f32) -> (f32, f32) {
        let scaling_factor = self.aspect_ratio_scale_factor();
        let tall_aspect_ratio = self.aspect_ratio() > self.logical_aspect_ratio();

        let logical_width_scaled = self.logical_width() as f32 * scaling_factor;
        let logical_height_scaled = self.logical_height() as f32 * scaling_factor;

        if tall_aspect_ratio {
            (x*scaling_factor + (self.screen_width() as f32 / 2.0) - (logical_width_scaled / 2.0),
             y*scaling_factor)
        } else {
            (x*scaling_factor,
             y*scaling_factor + (self.screen_height() as f32 / 2.0) - (logical_height_scaled / 2.0))
        }
    }

    fn scale_xy_pair_to_logical(&self, x: f32, y: f32) -> (f32, f32) {
        let scaling_factor = self.aspect_ratio_scale_factor();
        (x / scaling_factor, y / scaling_factor)
    }

    fn scale_xy_pair_to_real(&self, x: f32, y: f32) -> (f32, f32) {
        let scaling_factor = self.aspect_ratio_scale_factor();
        (x * scaling_factor, y * scaling_factor)
    }

    fn get_letterbox_viewport_rectangle(&self) -> (f32, f32, f32, f32) {
        let (x,y) = self.scale_and_transform_xy_pair_to_real(0.0, 0.0);
        let (w,h) = self.scale_xy_pair_to_real(self.logical_width() as f32, self.logical_height() as f32);
        // let (w,h) = self.resolution();
        (x,y,w,h)
    }

    pub fn font_size_percent(&self, percent: f32) -> u16 {
        (self.logical_height() as f32 * percent) as u16
    }

    // this is a literal text dimensions. This doesn't
    // account for the virtual resolution system.
    pub fn text_dimensions(&mut self, font_id: &str, text: &str, font_size: u16) -> (u32, u32) {
        if let Some(font_at_size) = self.find_text_asset_by_size(font_id, font_size) {
            let (width, height) = font_at_size.size_of(text).unwrap();
            (width, height)
        } else {
            (0, 0)
        }
    }

    pub fn scale_font_size(&self, font_size: u16) -> u16 {
        (font_size as f32 * self.aspect_ratio_scale_factor()) as u16
    }

    // This will get the virtual resolution
    pub fn logical_text_dimensions(&mut self, font_id: &str, text: &str, font_size: u16) -> (u32, u32) {
        let font_size = self.scale_font_size(font_size);
        let result = self.text_dimensions(font_id, text, font_size);
        let (w, h) = self.scale_xy_pair_to_logical(result.0 as f32, result.1 as f32);
        (w as u32, h as u32)
    }

    fn draw_string_texture(&mut self,
                           font_id: &str,
                           text: &str,
                           font_size: u16,
                           style: sdl2::ttf::FontStyle) -> Option<sdl2::render::Texture> {
        let texture_creator = self.window_canvas.texture_creator();
        match self.find_text_asset_by_size_mut(font_id, font_size) {
            Some(font) => {
                if font.get_style() != style {
                    font.set_style(style);
                }
                let font_surface = font.render(text)
                    .blended(SDLColor::RGBA(255, 255, 255, 255))
                    .expect("how did this go wrong?");

                let texture = texture_creator.create_texture_from_surface(
                    &font_surface
                ).expect("how did this go wrong?");
                std::mem::drop(font_surface);
                Some(texture)
            },
            None => { None }
        }
    }

    fn load_cached_string_texture(&mut self,
                                  font_id: &str,
                                  text: &str,
                                  font_size: u16,
                                  style: sdl2::ttf::FontStyle) -> Option<&mut sdl2::render::Texture> {
        // is &str hashable?
        let key =
            StaticTextCacheKey {
                font_id: font_id.to_owned(),
                text: text.to_owned(),
                font_size,
                style,
            };
        let is_already_in_cache = self.static_text_texture_cache.get(&key).is_some();

        if is_already_in_cache {
            self.static_text_texture_cache.get_mut(&key)
        } else {
            let new_texture_to_cache = self.draw_string_texture(font_id, text, font_size, style);
            match new_texture_to_cache {
                Some(texture) => {
                    #[cfg(debug_assertions)]
                    println!("inserting");
                    self.static_text_texture_cache.insert(key.clone(), texture);
                    self.static_text_texture_cache.get_mut(&key)
                },
                None => None
            }
        }
    }

    fn get_cached_string_texture(&self,
                                  font_id: &str,
                                  text: &str,
                                  font_size: u16,
                                  style: sdl2::ttf::FontStyle) -> Option<&sdl2::render::Texture> {
        // is &str hashable?
        let key =
            StaticTextCacheKey {
                font_id: font_id.to_owned(),
                text: text.to_owned(),
                font_size,
                style,
            };
        self.static_text_texture_cache.get(&key)
    }

    pub fn clear_font_cache(&mut self) {
        self.font_assets.clear();
    }

    pub fn clear_static_string_cache(&mut self) {
        {
            let drained = self.static_text_texture_cache.drain();
            for (_, texture) in drained {
                unsafe{ texture.destroy(); }
            }
        }
    }

    pub fn clear_resources(&mut self) {
        self.clear_font_cache();
        self.clear_static_string_cache();
    }

    pub fn render_static_text(&mut self,
                              font_id: &str,
                              x: f32,
                              y: f32,
                              text: &str,
                              font_size: u16,
                              color: Color,
                              style: sdl2::ttf::FontStyle) -> f32 {
        let font_size = self.scale_font_size((font_size as f32 * self.camera.scale) as u16);
        let (x, y) = self.scale_xy_pair_to_real((x * self.camera.scale) + self.camera.x,
                                                (y * self.camera.scale) + self.camera.y);

        {
            let mut text_texture = self.load_cached_string_texture(font_id, text, font_size, style);
            match &mut text_texture {
                Some(text_texture) => {
                    text_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                    text_texture.set_color_mod(color.r, color.g, color.b);
                    text_texture.set_alpha_mod(color.a);
                },
                None => {},
            }
        }
        let &mut SDL2GraphicsContext { ref mut window_canvas,
                                       ref static_text_texture_cache, .. } = self;
        {
            let text_texture = static_text_texture_cache.get(
                &StaticTextCacheKey {
                    font_id: font_id.to_owned(),
                    text: text.to_owned(),
                    font_size,
                    style,
                }
            ).unwrap();
            let sdl2::render::TextureQuery { width, height, .. } = text_texture.query();
            window_canvas.copy(&text_texture, None,
                               Some(sdl2::rect::Rect::new(x as i32,
                                                          y as i32,
                                                          width,
                                                          height))).unwrap();
            return self.scale_xy_pair_to_logical(width as f32, 0.0).0;
        }
    }

    pub fn render_text(&mut self,
                       font_id: &str,
                       x: f32,
                       y: f32,
                       text: &str,
                       font_size: u16,
                       color: Color,
                       style: sdl2::ttf::FontStyle) -> f32 {
        let font_size = self.scale_font_size((font_size as f32 * self.camera.scale) as u16);
        let (x, y) = self.scale_xy_pair_to_real((x * self.camera.scale) + self.camera.x,
                                                (y * self.camera.scale) + self.camera.y);

        match self.draw_string_texture(font_id, text, font_size, style) {
            Some(mut text_texture) => {
                text_texture.set_blend_mode(sdl2::render::BlendMode::Blend);
                text_texture.set_color_mod(color.r, color.g, color.b);
                text_texture.set_alpha_mod(color.a);

                let sdl2::render::TextureQuery { width, height, .. } = text_texture.query();
                self.window_canvas.copy(&text_texture, None,
                                        Some(sdl2::rect::Rect::new(x as i32,
                                                                   y as i32,
                                                                   width,
                                                                   height)))
                    .unwrap();
                unsafe{text_texture.destroy();}
                return self.scale_xy_pair_to_logical(width as f32, 0.0).0;
            },
            None => { panic!("!"); }
        }
    }

    pub fn render_text_justified(&mut self,
                                 font_id: &str,
                                 bounds: TextBounds,
                                 justification: TextJustification,
                                 text: &str,
                                 font_size: u16,
                                 color: Color,
                                 style: sdl2::ttf::FontStyle) -> f32 {
        let (width, height) = self.text_dimensions(font_id, text, font_size);
        let (x, y, w, h) = {
            match bounds {
                TextBounds::EntireScreen => (0.0, 0.0, self.logical_width() as f32, self.logical_height() as f32),
                TextBounds::Rectangle(x,y,w,h) => (x, y, w, h),
                TextBounds::ScreenLine(x, y) => (x, y, self.logical_width() as f32, font_size as f32),
            }
        };
        let x = {
            match justification.0 {
                TextJustificationHorizontal::Left => x,
                TextJustificationHorizontal::Right => ((w - width as f32) + x),
                TextJustificationHorizontal::Center => (w / 2.0) - (width as f32 / 2.0) + x,
            }
        };
        let y = {
            match justification.1 {
                TextJustificationVertical::Up => x,
                TextJustificationVertical::Down => ((h - height as f32) + y),
                TextJustificationVertical::Center => (h / 2.0) - (height as f32 / 2.0) + y,
            }
        };
        self.render_text(font_id, x, y, text, font_size, color, style)
    }

    pub fn render_filled_rectangle(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let (x, y) = self.scale_xy_pair_to_real((x * self.camera.scale) + self.camera.x,
                                                (y * self.camera.scale) + self.camera.y);
        let (w, h) = self.scale_xy_pair_to_real(w * self.camera.scale, h * self.camera.scale);
        let white_rectangle = &mut self.white_rectangle_texture;
        white_rectangle.set_blend_mode(sdl2::render::BlendMode::Blend);
        white_rectangle.set_color(color);

        self.window_canvas.copy(&white_rectangle.texture, None,
                                Some(sdl2::rect::Rect::new(x as i32,
                                                           y as i32,
                                                           w as u32,
                                                           h as u32)))
            .unwrap();
    }
}
