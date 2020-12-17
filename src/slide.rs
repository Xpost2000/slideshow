// TODO
use crate::utility::*;
use crate::color::*;

#[derive(Debug, Clone)]
pub struct TextElement {
    /*font?*/
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub line_breaks: u32,
    pub text: String, // In the case I allow variables or something...
    pub color: Color,
    pub font_size: u16,
    pub font_name: Option<String>,
}
#[derive(Debug, Clone)]
pub struct ImageElement {
    pub background: bool, // whether it affects layout...
    pub location: String,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub w: Option<f32>,
    pub h: Option<f32>,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum SlideElement {
    Text(TextElement),
    Image(ImageElement),
}

#[derive(Debug,Copy,Clone)]
pub enum SlideTransitionType {
    HorizontalSlide,
    VerticalSlide,
    FadeTo(Color),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Page {
    pub transition : Option<SlideTransition>,

    pub background_color: Color,
    pub elements: Vec<SlideElement>,
}

use crate::graphics_context::*;

impl Page {
    pub fn render(&self,
                  graphics_context: &mut SDL2GraphicsContext,
                  default_font: &str) {
        use crate::markup::*;
        graphics_context.render_filled_rectangle(0.0, 0.0,
                                                 graphics_context.logical_width() as f32,
                                                 graphics_context.logical_height() as f32,
                                                 self.background_color);

        let mut last_font_size : u16 = 0;
        let mut cursor_x : f32 = 0.0;
        let mut cursor_y : f32 = 0.0;

        let mut cursor_x_baseline: f32 = 0.0;
        let mut cursor_y_baseline: Option<f32> = None;

        for element in &self.elements {
            match element {
                SlideElement::Text(text) => {
                    let font_size = text.font_size;
                    cursor_x_baseline = match text.x {
                        Some(x) => x,
                        None => 0.0,
                    };

                    cursor_x = cursor_x_baseline;

                    if let Some(baseline_y) = text.y {
                        if cursor_y_baseline.is_none() {
                            cursor_y_baseline = Some(baseline_y);
                            cursor_y = cursor_y_baseline.unwrap();
                        } else {
                            if baseline_y != cursor_y_baseline.unwrap() {
                                cursor_y_baseline = Some(baseline_y);
                                cursor_y = cursor_y_baseline.unwrap();
                            }
                        }
                    }

                    let markup_lexer = MarkupLexer::new(&text.text);
                    let drawn_font =
                        if let Some(font) = &text.font_name {
                            graphics_context.add_font(font)
                        } else {
                            default_font 
                        };

                    let height = graphics_context.text_dimensions(drawn_font, &text.text, font_size).1;
                    if last_font_size == 0 { last_font_size = height as u16; }
                    cursor_y += last_font_size as f32 * text.line_breaks as f32;

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
                        cursor_x += width as f32;
                    }
                    cursor_y += height as f32;
                    last_font_size = font_size;
                    cursor_x = cursor_x_baseline;
                },
                SlideElement::Image(image) => {
                    let texture = graphics_context.add_image(&image.location);
                    let image_dimensions = graphics_context.image_dimensions(texture);

                    if let Some(baseline_y) = image.y {
                        if cursor_y_baseline.is_none() {
                            cursor_y_baseline = Some(baseline_y);
                            cursor_y = cursor_y_baseline.unwrap();
                        } else {
                            if baseline_y != cursor_y_baseline.unwrap() {
                                cursor_y_baseline = Some(baseline_y);
                                cursor_y = cursor_y_baseline.unwrap();
                            }
                        }
                    }

                    if let Some(x) = image.x {
                        cursor_x = x;
                    }

                    let image_width = match image.w {
                        Some(w) => w,
                        None => image_dimensions.0 as f32,
                    };
                    let image_height = match image.h {
                        Some(h) => h,
                        None => image_dimensions.1 as f32,
                    };
                    graphics_context.render_image(texture,
                                                  cursor_x,
                                                  cursor_y,
                                                  image_width,
                                                  image_height,
                                                  image.color);

                    if !image.background {
                        cursor_y +=
                            match image.y {
                                None => image_height,
                                Some(y) => y,
                            };
                    }
                },
                _ => unimplemented!("????")
            }
        }
    }
}

impl Default for Page {
    fn default() -> Page {
        Page {
            transition: None,
            background_color: COLOR_WHITE,
            elements: Vec::new()
        }
    }
}

pub struct Slide {
    pub file_name : String, // owned string for hot reloading.
    pub last_modified_time: std::time::SystemTime,

    pub pages : Vec<Page>,
    pub current_page : isize,

    pub resolution : (u32, u32),
}
impl Default for Slide {
    fn default() -> Slide {
        Slide {
            file_name: String::new(),
            pages: Vec::new(),
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

        if let Some(selected_page) = self.get(page) {
            graphics_context.use_viewport_letterbox();
            selected_page.render(graphics_context, default_font);
        } else {
            graphics_context.clear_color(Color::new(10, 10, 16, 255));
            graphics_context.use_viewport_default();

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
                    *self = slide;
                    self.current_page = 0;
                }
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn finish_transition(&mut self) {
        if let Some(current_page) = &mut self.get_current_page() {
            if let Some(transition) = &mut self.get_current_page_mut().unwrap().transition {
                transition.time = 0.0;
            }
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

    pub fn get_current_page_mut(&mut self) -> Option<&mut Page> {
        self.get_mut(self.current_page as usize)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Page> {
        self.pages.get_mut(index)
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

