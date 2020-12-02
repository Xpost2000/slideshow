// TODO
use std::hash::{Hash, Hasher};
use crate::utility::*;
use crate::color::*;

#[derive(Debug, Clone)]
pub struct TextElement {
    /*font?*/
    pub x: f32,
    pub y: f32, // TODO: Rename to linebreaks between. Or something
    pub text: String, // In the case I allow variables or something...
    pub color: Color,
    pub font_size: u16,
    pub font_name: Option<String>,
}
impl Hash for TextElement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        fn canonicalize_position(e: &TextElement) -> (i32, i32) {
            (e.x.round() as i32, e.y.round() as i32)
        }

        canonicalize_position(self).hash(state);
        self.text.hash(state);
        self.color.hash(state);
        self.font_size.hash(state);
        self.font_name.hash(state);
    }
}
#[derive(Debug, Hash, Clone)]
pub struct Page {
    pub background_color: Color,
    /*
    A Page is probably just going to consist of "elements"
    
    Like a TextElement
    and ImageElement
    and ShapeElement
    or whatever primitives I want to add...
     */
    pub text_elements: Vec<TextElement>,
}

use crate::graphics_context::*;

impl Page {
    fn calculate_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher_state = DefaultHasher::new();
        self.hash(&mut hasher_state);
        hasher_state.finish()
    }

    pub fn render(&self,
                  graphics_context: &mut SDL2GraphicsContext,
                  default_font: &str) {
        use crate::markup::*;
        graphics_context.render_filled_rectangle(0.0, 0.0,
                                                 graphics_context.logical_width() as f32,
                                                 graphics_context.logical_height() as f32,
                                                 self.background_color);

        let mut last_font_size : u16 = 0;
        let mut cursor_y : f32 = 0.0;

        for text in &self.text_elements {
            let font_size = text.font_size;
            let mut cursor_x : f32 = 0.0;

            let markup_lexer = MarkupLexer::new(&text.text);
            let drawn_font =
                if let Some(font) = &text.font_name {
                    graphics_context.add_font(font)
                } else {
                    default_font 
                };

            let height = graphics_context.text_dimensions(drawn_font, &text.text, font_size).1;
            if last_font_size == 0 { last_font_size = height as u16; }
            cursor_y += last_font_size as f32 * text.y;

            for markup in markup_lexer {
                let text_content = markup.get_text_content();
                let width = graphics_context.render_static_text(drawn_font,
                                                                cursor_x, cursor_y,
                                                                text_content,
                                                                font_size,
                                                                text.color,
                                                                markup.get_text_drawing_style());
                // render decoration
                match markup {
                    Markup::Strikethrough(_) => {
                        graphics_context.render_filled_rectangle(cursor_x,
                                                                 cursor_y + (font_size as f32 / 1.8),
                                                                 width as f32,
                                                                 font_size as f32 / 10.0,
                                                                 text.color);
                    }
                    Markup::Underlined(_) => {
                        graphics_context.render_filled_rectangle(cursor_x,
                                                                 cursor_y + (font_size as f32),
                                                                 width as f32,
                                                                 font_size as f32 / 13.0,
                                                                 text.color);
                    }
                    _ => {},
                }
                cursor_x += (width) as f32;
            }

            cursor_y += height as f32;
            last_font_size = font_size;
        }
    }
}

impl Default for Page {
    fn default() -> Page {
        Page {
            background_color: COLOR_WHITE,
            text_elements: Vec::new()
        }
    }
}

#[derive(Debug,Copy,Clone)]
pub enum SlideTransitionType {
    HorizontalSlide,
    VerticalSlide,
    FadeTo(Color),
}

#[derive(Debug)]
pub struct SlideTransition {
    pub transition_type: SlideTransitionType, // type is keyword :(
    pub easing_function: EasingFunction,
    pub time: f32,
    pub finish_time: f32,
}
impl SlideTransition {
    pub fn finished_fraction(&self) -> f32 {
        self.time / self.finish_time
    }
    pub fn finished_transition(&self) -> bool {
        self.time >= self.finish_time
    }
    pub fn easing_amount(&self) -> f32 {
        self.easing_function.evaluate(0.0, 1.0, self.time/self.finish_time)
    }
}
pub struct Slide {
    pub file_name : String, // owned string for hot reloading.
    pub last_modified_time: std::time::SystemTime,

    pub pages : Vec<Page>,
    pub current_page : isize,

    pub transition : Option<SlideTransition>,
    pub resolution : (u32, u32),
}
impl Default for Slide {
    fn default() -> Slide {
        Slide {
            file_name: String::new(),
            pages: Vec::new(),
            // transition: None,
            transition: Some(
                SlideTransition {
                    // transition_type: SlideTransitionType::HorizontalSlide,
                    transition_type: SlideTransitionType::FadeTo(COLOR_BLACK),
                    easing_function: EasingFunction::Linear,
                    time: 0.0,
                    finish_time: 1.0
                }),
            current_page: isize::default(),
            last_modified_time: std::time::SystemTime::now(),// eh...
            resolution: (1280, 720),
        }
    }
}

impl Slide {
    pub fn new_from_file(file_name: &str) -> Option<Slide> {
        match load_file(file_name) {
            Ok(file_source) => {
                use crate::slide_parser::compile_slide;
                let slideshow_source = remove_comments_from_source(&file_source);
                let new_slide = Slide {
                    file_name: file_name.to_owned(),
                    current_page: 0,
                    last_modified_time: file_last_modified_time(file_name),
                    .. compile_slide(&slideshow_source)
                };

                Some(new_slide)
            },
            Err(_) => {
                None
            }
        }
    }

    pub fn try_to_draw_page(&self,
                            graphics_context: &mut SDL2GraphicsContext,
                            default_font: &str,
                            page: usize) {
        graphics_context.logical_resolution = VirtualResolution::Virtual(self.resolution().0,
                                                                         self.resolution().1);
        graphics_context.use_viewport_letterbox();

        if let Some(selected_page) = self.get(page) {
            selected_page.render(graphics_context, default_font);
        } else {
            graphics_context.clear_color(Color::new(10, 10, 16, 255));
            graphics_context.logical_resolution = VirtualResolution::Display;
            graphics_context.render_text_justified(default_font,
                                                   TextBounds::EntireScreen,
                                                   TextJustification::center(),
                                                   "stupid slide needs pages... feed me!",
                                                   graphics_context.font_size_percent(0.073),
                                                   COLOR_WHITE,
                                                   sdl2::ttf::FontStyle::NORMAL);
        }
    }

    // handle errors more explictly...
    #[allow(dead_code)]
    pub fn file_last_modified_time(&self) -> std::time::SystemTime {
        file_last_modified_time(&self.file_name)
    }
    /*
    This will load and keep the current page around something with a similar hash.
    If it has the same page count it will reliably return the same page but modified.

    The hash idea isn't very great, but might be okay if I replace it with a similarity
    score system or something.
     */
    pub fn reload(&mut self) -> Result<(), ()> {
        let previous_page_count = self.len();
        let previous_current_page = self.current_page();

        let new_slide = Slide::new_from_file(&self.file_name);
        if let Some(slide) = new_slide {
            if slide.last_modified_time > self.last_modified_time {
                if previous_page_count == slide.len() {
                    *self = slide;
                    self.current_page = previous_current_page;
                } else {
                    let current_page_hash = self.get_current_page()
                        .expect("should have page...").calculate_hash();
                    let mut hash_delta: u64 = std::u64::MAX;
                    let mut closest_hash_and_index: (u64, usize) = (0, 0);
                    for (index, page) in slide.pages.iter().enumerate() {
                        let page_hash = page.calculate_hash();
                        if page_hash == current_page_hash {
                            closest_hash_and_index = (current_page_hash, index);
                            break;
                        } else {
                            let min = std::cmp::min(page_hash, current_page_hash);
                            let max = std::cmp::max(page_hash, current_page_hash);
                            if (max - min) < hash_delta {
                                hash_delta = max - min;
                                closest_hash_and_index = (page_hash, index);
                            }
                        }
                    }
                    *self = slide;
                    self.current_page = closest_hash_and_index.1 as isize;
                }
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn finish_transition(&mut self) {
        if let Some(transition) = &mut self.transition {
            transition.time = 0.0;
        }
    }

    pub fn resolution(&self) -> (u32, u32) {
        self.resolution
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }
    pub fn current_page(&self) -> isize {
        self.current_page
    }
    pub fn get_current_page(&self) -> Option<&Page> {
        self.get(self.current_page as usize)
    }

    pub fn get(&self, index: usize) -> Option<&Page> {
        self.pages.get(index)
    }

    pub fn next_page(&mut self) -> isize {
        let desired_next_page = self.current_page + 1;
        self.current_page += 1;
        self.current_page = clamp(self.current_page as i32, 0, self.len() as i32 - 1) as isize;
        desired_next_page
    }

    pub fn previous_page(&mut self) -> isize {
        let desired_next_page = self.current_page - 1;
        self.current_page -= 1;
        self.current_page = clamp(self.current_page as i32, 0, self.len() as i32 - 1) as isize;
        desired_next_page
    }
}

