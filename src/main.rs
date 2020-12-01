/*
    beginnings of a slideshow program?

TODO: Please rewrite the tokenizer.
*/
mod markup;
use self::markup::*;
mod utility;
use self::utility::*;
mod graphics_context;
use self::graphics_context::*;
mod color;
use self::color::*;
mod slide_parser;
use self::slide_parser::*;
mod slide;
use self::slide::*;

use sdl2::event::Event as SDLEvent;
use sdl2::keyboard::Keycode as SDLKeycode;

const DEFAULT_WINDOW_WIDTH : u32 = 1024;
const DEFAULT_WINDOW_HEIGHT : u32 = 768;
const DEFAULT_SLIDE_WHEN_NONE_GIVEN : &'static str = "test.slide";

#[derive(Clone, Copy)]
enum ApplicationScreen {
    InvalidOrNoSlide,
    Options,
    ShowingSlide,
    SelectSlideToLoad,
    ChangePage(isize, isize),
    Quit,
}

const DEFAULT_LAST_WRITE_TIMER_INTERVAL : f32 = 0.20;
struct ApplicationState {
    state: ApplicationScreen,

    // TODO: Handle external drives?
    // That's a pretty big deal actually.
    current_working_directory: std::path::PathBuf,
    // options state,
    currently_selected_resolution: usize,
    currently_selected_directory: usize,
    last_write_timer: f32,
    // everything else I guess
    slideshow: Option<Slide>,
}

impl ApplicationState {
    fn new(command_line_arguments: &Vec<String>) -> ApplicationState {
        ApplicationState {
            state: ApplicationScreen::ShowingSlide,
            current_working_directory: std::path::PathBuf::from("./").canonicalize().unwrap(),
            currently_selected_resolution: 0,
            currently_selected_directory: 0,
            last_write_timer: 0.0,

            slideshow:
            Slide::new_from_file(
                match command_line_arguments.len() {
                    1 => {
                        DEFAULT_SLIDE_WHEN_NONE_GIVEN
                    }
                    2 => {
                        &command_line_arguments[1]
                    },
                    _ => {
                        println!("The only command line argument should be the slide file!");
                        DEFAULT_SLIDE_WHEN_NONE_GIVEN
                    }
                }
            )
        }
    }

    fn update(&mut self, delta_time: f32) {
        // TODO better timer and make it read based on last write/modified time...
        if let Some(slideshow) = &mut self.slideshow {
            if self.last_write_timer <= 0.0 {
                self.last_write_timer = DEFAULT_LAST_WRITE_TIMER_INTERVAL;
                slideshow.reload();
            }
        }
        self.last_write_timer -= delta_time;

        match self.state {
            ApplicationScreen::Quit | ApplicationScreen::InvalidOrNoSlide |
            ApplicationScreen::Options => {},
            ApplicationScreen::SelectSlideToLoad => {
            },
            ApplicationScreen::ShowingSlide => {
                if let None = &self.slideshow {
                    self.state = ApplicationScreen::InvalidOrNoSlide;
                }
            },
            ApplicationScreen::ChangePage(first, second) => {
                if let Some(slideshow) = &mut self.slideshow {
                    let valid_transition = slideshow.get(first as usize).is_some() && slideshow.get(second as usize).is_some();
                    if let None = slideshow.transition {
                        self.state = ApplicationScreen::ShowingSlide;
                    } else if let Some(transition) = &mut slideshow.transition {
                        if valid_transition && !transition.finished_transition() {
                            transition.time += delta_time;
                        } else {
                            self.state = ApplicationScreen::ShowingSlide;
                            transition.time = 0.0;
                        }
                    }
                }
            },
        }
    }

    fn draw(&self, graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");

        match self.state {
            ApplicationScreen::Quit => {},
            /*
            TODO: I can see a way of refactoring this out
            into a menu trait that depends on something that can turn into
            an iterator.
            
            I don't need a real GUI or anything so this honestly works just fine, and probably just looks
            plain nicer to have.
             */
            ApplicationScreen::SelectSlideToLoad => {
                graphics_context.logical_resolution = VirtualResolution::Display;
                graphics_context.clear_color(Color::new(10, 10, 16, 255));
                let heading_font_size = graphics_context.font_size_percent(0.08);
                let (width, heading_height) = graphics_context.text_dimensions(default_font, "Browse For Slide File(left arrow, for back)", heading_font_size);
                graphics_context.render_text(default_font,
                                             ((graphics_context.logical_width() as i32 / 2) - (width as i32) / 2) as f32,
                                             0.0,
                                             "Browse For Slide File(left arrow, for back)",
                                             heading_font_size,
                                             COLOR_WHITE,
                                             sdl2::ttf::FontStyle::NORMAL);

                let directory_listing = std::fs::read_dir(&self.current_working_directory).expect("Failed to get directory listing?");

                graphics_context.render_text(default_font,
                                             ((graphics_context.logical_width() as i32 / 2) - (width as i32) / 2) as f32,
                                             heading_height as f32,
                                             #[cfg(target_os = "windows")]
                                             &format!("{}", &self.current_working_directory.to_str().unwrap())[4..],
                                             #[cfg(target_os = "unix")]
                                             &format!("{}", &self.current_working_directory.to_str().unwrap()),
                                             heading_font_size,
                                             Color::new(128, 128, 128, 255),
                                             sdl2::ttf::FontStyle::NORMAL);

                let mut draw_cursor_y : f32 = (heading_height*2) as f32;
                let listings_to_show = 10;
                // TODO: refactor
                graphics_context.render_filled_rectangle((((graphics_context.logical_width() as i32 / 2)) - 250) as f32,
                                                         draw_cursor_y-10.0,
                                                         (graphics_context.logical_width()/2) as f32,
                                                         listings_to_show as f32 * graphics_context.font_size_percent(0.06) as f32,
                                                         Color::new(5, 5, 8, 255));
                let directory_listing = directory_listing.into_iter();
                for (index, path) in directory_listing.
                    skip(self.currently_selected_directory)
                    .take(listings_to_show)
                    .enumerate() {
                        let is_selected = (index == 0);
                        let directory_string = {
                            let path = path.as_ref().expect("bad permission?").path();
                            let path_name = path.file_name().unwrap() .to_str().unwrap();

                            if is_selected {
                                format!("* {}", path_name)
                            } else {
                                format!("{}", path_name)
                            }
                        };

                        let font_size = graphics_context.font_size_percent(
                            if is_selected {
                                0.053
                            } else {
                                0.045
                            });
                        let (width, height) = graphics_context.text_dimensions(default_font, &directory_string, font_size);

                        graphics_context.render_text(default_font,
                                                     (((graphics_context.logical_width() as i32 / 2)) - 250) as f32,
                                                     draw_cursor_y,
                                                     &directory_string,
                                                     font_size,
                                                     if is_selected {
                                                         COLOR_RIPE_LEMON
                                                     } else {
                                                         COLOR_WHITE
                                                     },
                                                     sdl2::ttf::FontStyle::NORMAL);
                        draw_cursor_y += height as f32;
                    }
            },
            ApplicationScreen::InvalidOrNoSlide => {
                graphics_context.clear_color(Color::new(10, 10, 16, 255));
                graphics_context.logical_resolution = VirtualResolution::Display;
                graphics_context.render_text_justified(default_font,
                                                       TextBounds::EntireScreen,
                                                       TextJustification::center(),
                                                       "Invalid / No slide file",
                                                       graphics_context.font_size_percent(0.073),
                                                       COLOR_WHITE,
                                                       sdl2::ttf::FontStyle::NORMAL);
            },
            ApplicationScreen::Options => {
                graphics_context.logical_resolution = VirtualResolution::Display;
                graphics_context.clear_color(Color::new(10, 10, 16, 255));
                let heading_font_size = graphics_context.font_size_percent(0.08);
                graphics_context.render_text_justified(default_font,
                                                       TextBounds::ScreenLine(0.0, 0.0),
                                                       TextJustification::center(),
                                                       "Resolution Select",
                                                       heading_font_size,
                                                       COLOR_WHITE,
                                                       sdl2::ttf::FontStyle::NORMAL);
                let resolutions = graphics_context.get_avaliable_resolutions();
                let resolution_count = resolutions.iter().count();
                let resolutions_to_show = 8; 

                let mut draw_cursor_y : f32 = (heading_font_size*2) as f32;
                for (index, resolution) in resolutions .iter()
                    .skip(self.currently_selected_resolution)
                    .take(resolutions_to_show).enumerate() {
                    let is_selected = index == 0;
                    let resolution_string =
                        if is_selected {
                            format!("* {} x {}", resolution.0, resolution.1)
                        } else {
                            format!("{} x {}", resolution.0, resolution.1)
                        };
                    let font_size =
                        if is_selected {
                            graphics_context.font_size_percent(0.073)
                        } else {
                            graphics_context.font_size_percent(0.057)
                        };
                    graphics_context.render_text_justified(default_font,
                                                           TextBounds::ScreenLine(0.0, draw_cursor_y),
                                                           TextJustification::center(),
                                                           &resolution_string,
                                                           font_size,
                                                           if is_selected {
                                                               COLOR_RIPE_LEMON 
                                                           } else {
                                                               COLOR_WHITE
                                                           },
                                                           sdl2::ttf::FontStyle::NORMAL);
                    draw_cursor_y += font_size as f32;
                }
            },
            ApplicationScreen::ChangePage(first, second) => {
                #[cfg(debug_assertions)]
                graphics_context.clear_color(Color::new(255, 0, 0, 255));
                let slideshow = &self.slideshow.as_ref().unwrap();
                if let Some(transition) = &slideshow.transition {
                    let easing_amount = transition.easing_amount();
                    let forward_direction = second > first;
                    match transition.transition_type {
                        // These two transitions are almost identical... maybe I should refactor this later.
                        SlideTransitionType::HorizontalSlide => {
                            graphics_context.camera.y = 0.0;

                            graphics_context.camera.x =
                                if forward_direction {
                                    0.0 - graphics_context.logical_width() as f32 * easing_amount
                                } else {
                                    0.0 + graphics_context.logical_width() as f32 * easing_amount
                                };
                            slideshow.get(first as usize)
                                .unwrap()
                                .render(graphics_context, default_font);
                            graphics_context.camera.x =
                                if forward_direction {
                                    graphics_context.logical_width() as f32 - (graphics_context.logical_width() as f32 * easing_amount)
                                } else {
                                    -(graphics_context.logical_width() as f32) + (graphics_context.logical_width() as f32 * easing_amount)
                                };
                            slideshow.get(second as usize)
                                .unwrap()
                                .render(graphics_context, default_font);
                        },
                        SlideTransitionType::VerticalSlide => {
                            graphics_context.camera.x = 0.0;

                            graphics_context.camera.y =
                                if forward_direction {
                                    0.0 - graphics_context.logical_height() as f32 * easing_amount
                                } else {
                                    0.0 + graphics_context.logical_height() as f32 * easing_amount
                                };
                            slideshow.get(first as usize)
                                .unwrap()
                                .render(graphics_context, default_font);
                            graphics_context.camera.y =
                                if forward_direction {
                                    graphics_context.logical_height() as f32 - (graphics_context.logical_height() as f32 * easing_amount)
                                } else {
                                    -(graphics_context.logical_height() as f32) + (graphics_context.logical_height() as f32 * easing_amount)
                                };
                            slideshow.get(second as usize)
                                .unwrap()
                                .render(graphics_context, default_font);
                        },
                        SlideTransitionType::FadeTo(color) => {
                            // split time into two halves.
                            let half_max_time = (transition.finish_time / 2.0);
                            let ease_function = transition.easing_function;
                            let fraction_of_completion = transition.finished_fraction();
                            let (alpha, page_to_draw) = {
                                let ease_amount =
                                    if fraction_of_completion < 0.5 {
                                        ease_function.evaluate(0.0, 1.0, transition.time/half_max_time)
                                    } else {
                                        ease_function.evaluate(1.0, 0.0, (transition.time-half_max_time)/half_max_time)
                                    };
                                let alpha = 255 as f32 * ease_amount;
                                (
                                    clamp(alpha, 0.0, 255.0) as u8,
                                    if fraction_of_completion < 0.5 {
                                        first
                                    } else {
                                        second
                                    }
                                )
                            };
                            let color = Color{a: alpha, .. color};
                            slideshow.get(page_to_draw as usize)
                                .unwrap()
                                .render(graphics_context, default_font);
                            graphics_context.render_filled_rectangle(0.0, 0.0,
                                                                     graphics_context.logical_width() as f32,
                                                                     graphics_context.logical_height() as f32,
                                                                     color);
                        },
                    }
                }
            },
            ApplicationScreen::ShowingSlide => {
                if let Some(slideshow) = &self.slideshow {
                    graphics_context.camera.x = 0.0;
                    graphics_context.camera.y = 0.0;
                    graphics_context.clear_color(Color::new(0, 0, 0, 255));
                    if let Some(current_slide) = slideshow.get_current_page() {
                        current_slide.render(graphics_context, default_font);
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
            },
        }
    }

    fn handle_input(&mut self, graphics_context: &mut SDL2GraphicsContext, event_pump: &mut sdl2::EventPump, delta_time: f32) {
        match self.state {
            ApplicationScreen::Quit => {},
            ApplicationScreen::SelectSlideToLoad => {
                let directory_listing = std::fs::read_dir(&self.current_working_directory).expect("Failed to get directory listing?");
                self.currently_selected_directory = clamp(self.currently_selected_directory,
                                                          0, directory_listing.into_iter().count()-1);
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Return), .. } |
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. }  =>  {
                            let directory_listing = std::fs::read_dir(&self.current_working_directory).expect("Failed to get directory listing?");
                            let selected_path = directory_listing.into_iter().nth(self.currently_selected_directory);

                            if let Some(path) = selected_path {
                                let path = path.expect("bad permission?");
                                let file_type = path.file_type().unwrap();

                                if file_type.is_dir() {
                                    self.current_working_directory = path.path();
                                    self.currently_selected_directory = 0;
                                } else {
                                    let new_slide = Slide::new_from_file(path.path().to_str().expect("bad unicode"));
                                    self.slideshow = new_slide;

                                    self.state = ApplicationScreen::ShowingSlide;
                                }
                            }
                        },

                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                            self.current_working_directory.pop();
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                            if self.currently_selected_directory > 0 {
                                self.currently_selected_directory -= 1;
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                            self.currently_selected_directory += 1;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                            self.state = ApplicationScreen::Options;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                            self.state = ApplicationScreen::ShowingSlide;
                        },
                        _ => {}
                    }
                }
            },
            ApplicationScreen::InvalidOrNoSlide => {
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                            self.state = ApplicationScreen::Options;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                            self.state = ApplicationScreen::SelectSlideToLoad;
                        },
                        _ => {}
                    }
                }
            },
            ApplicationScreen::Options => {
                let resolutions = graphics_context.get_avaliable_resolutions();
                let resolution_count = resolutions.iter().count();
                self.currently_selected_resolution = clamp(self.currently_selected_resolution,
                                                           0, resolution_count - 1);
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::F), ..} => {
                            graphics_context.toggle_fullscreen();
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Return), .. } =>  {
                            let resolution_list = graphics_context.get_avaliable_resolutions();
                            if let Some(resolution_pair) = resolution_list.get(self.currently_selected_resolution) {
                                graphics_context.set_resolution((resolution_pair.0 as u32, resolution_pair.1 as u32));
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                            if self.currently_selected_resolution > 0 {
                                self.currently_selected_resolution -= 1;
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                            self.currently_selected_resolution += 1;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                            self.state = ApplicationScreen::ShowingSlide;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                            self.state = ApplicationScreen::SelectSlideToLoad;
                        },
                        _ => {}
                    }
                }
            },
            ApplicationScreen::ChangePage(_, _) => {
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown {..} => {
                            self.state = ApplicationScreen::ShowingSlide;
                            if let Some(slideshow) = &mut self.slideshow {
                                slideshow.finish_transition();
                            }
                        },
                        _ => {}
                    }
                }
            }
            ApplicationScreen::ShowingSlide => {
                for event in event_pump.poll_iter() {
                    match event {
                        SDLEvent::Quit {..} => {
                            self.state = ApplicationScreen::Quit;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                            self.slideshow = None;
                            self.state = ApplicationScreen::InvalidOrNoSlide;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::F), ..} => {
                            graphics_context.toggle_fullscreen();
                        },
                        #[cfg(debug_assertions)]
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Num0), .. } => {
                            graphics_context.camera.scale = 1.0;
                        },
                        #[cfg(debug_assertions)]
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                            graphics_context.camera.scale += 1.0 * delta_time;
                        },
                        #[cfg(debug_assertions)]
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                            graphics_context.camera.scale -= 1.0 * delta_time;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::R), .. } => {
                            if let Some(slideshow) = &mut self.slideshow {
                                slideshow.reload();
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. } => {
                            if let Some(slideshow) = &mut self.slideshow {
                                self.state = ApplicationScreen::ChangePage(slideshow.current_page(),
                                                                           slideshow.next_page());
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                            if let Some(slideshow) = &mut self.slideshow {
                                self.state = ApplicationScreen::ChangePage(slideshow.current_page(),
                                                                           slideshow.previous_page());
                            }
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                            self.state = ApplicationScreen::SelectSlideToLoad;
                        },
                        SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                            self.state = ApplicationScreen::Options;
                        },
                        _ => {}
                    }
                }
            },
        }
    }
}

fn main() {
    let sdl2_context = sdl2::init().expect("SDL2 failed to initialize?");
    let video_subsystem = sdl2_context.video().unwrap();

    let sdl2_ttf_context = sdl2::ttf::init()
        .expect("SDL2 ttf failed to initialize?");
    let sdl2_image_context = sdl2::image::init(sdl2::image::InitFlag::PNG | sdl2::image::InitFlag::JPG)
        .expect("SDL2 image failed to initialize?");

    let window = video_subsystem.window("stupid slideshow", DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
        .position_centered()
        // .resizable()
        .build()
        .expect("Window failed to open?");

    let mut graphics_context = SDL2GraphicsContext::new(window,
                                                        &sdl2_ttf_context,
                                                        &sdl2_image_context,
                                                        &video_subsystem);
    let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");
    // let dumb_test_texture = graphics_context.add_image("data/res/rust-logo-png-transparent.png");
    let resolutions = graphics_context.get_avaliable_resolutions();

    let mut event_pump = sdl2_context.event_pump().unwrap();

    use std::env;
    let arguments : Vec<String> = env::args().collect();
    let mut application_state = ApplicationState::new(&arguments);

    let mut sdl2_timer = sdl2_context.timer().unwrap();
    let mut delta_time = 0;
    graphics_context.enable_alpha_blending();
    'running: loop {
        let start_time = sdl2_timer.ticks();
        graphics_context.clear_color(Color::new(0, 0, 0, 255));
        graphics_context.use_viewport_default();
        if let ApplicationScreen::Quit = application_state.state {
            break 'running;
        } else {
            let delta_time = delta_time as f32 / 1000.0;

            application_state.handle_input(&mut graphics_context, &mut event_pump, delta_time);
            application_state.update(delta_time);
            application_state.draw(&mut graphics_context);
        }
        graphics_context.present();
        let end_time = sdl2_timer.ticks();
        delta_time = end_time - start_time;
        #[cfg(debug_assertions)]
        {println!("{}", 1.0/(delta_time as f32/1000.0));}
    }
}
