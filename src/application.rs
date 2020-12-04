pub use crate::utility::*;

pub use crate::slide::*;
pub use crate::application_states::*;

const DEFAULT_LAST_WRITE_TIMER_INTERVAL : f32 = 0.20;

const DEFAULT_SLIDE_WHEN_NONE_GIVEN : &'static str = "test.slide";

pub struct ApplicationState {
    pub state: ApplicationScreen,

    pub current_working_directory: std::path::PathBuf,
    pub currently_selected_resolution: usize,
    pub currently_selected_directory: usize,
    pub last_write_timer: f32,
    pub slideshow: Option<Slide>,
}

impl ApplicationState {
    pub fn new(command_line_arguments: &Vec<String>) -> ApplicationState {
        ApplicationState {
            state: ApplicationScreen::ShowingSlide(ShowingSlideState),
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

    pub fn update(&mut self, delta_time: f32) {
        // TODO better timer and make it read based on last write/modified time...
        if let Some(slideshow) = &mut self.slideshow {
            if self.last_write_timer <= 0.0 {
                self.last_write_timer = DEFAULT_LAST_WRITE_TIMER_INTERVAL;
                slideshow.reload().expect("should be successful.");
            }
        }
        self.last_write_timer -= delta_time;

        match self.state {
            ApplicationScreen::Quit(_) | ApplicationScreen::InvalidOrNoSlide(_) |
            ApplicationScreen::Options(_) => {},
            ApplicationScreen::SelectSlideToLoad(_) => {
            },
            ApplicationScreen::ShowingSlide(_) => {
                if let None = &self.slideshow {
                    self.state = ApplicationScreen::InvalidOrNoSlide(InvalidOrNoSlideState);
                }
            },
            ApplicationScreen::ChangePage(ChangePageState{from, to}) => {
                let first = from;
                let second = to;
                if let Some(slideshow) = &mut self.slideshow {
                    let valid_transition = slideshow.get(first as usize).is_some() && slideshow.get(second as usize).is_some();
                    if let None = slideshow.transition {
                        self.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                    } else if let Some(transition) = &mut slideshow.transition {
                        if valid_transition && !transition.finished_transition() {
                            transition.time += delta_time;
                        } else {
                            self.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                            transition.time = 0.0;
                        }
                    }
                }
            },
        }
    }

    pub fn draw(&self, graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");

        match self.state {
            ApplicationScreen::Quit(_) => {},
            /*
            TODO: I can see a way of refactoring this out
            into a menu trait that depends on something that can turn into
            an iterator.
            
            I don't need a real GUI or anything so this honestly works just fine, and probably just looks
            plain nicer to have.
             */
            ApplicationScreen::SelectSlideToLoad(_) => {
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
                                             #[cfg(target_os = "windows")] // Weird thing that looks like a drive root? ?//?DRIVE_LETTER:/
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
                        let is_selected = index == 0;
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
                        let height = graphics_context.text_dimensions(default_font, &directory_string, font_size).1;

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
            ApplicationScreen::InvalidOrNoSlide(_) => {
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
            ApplicationScreen::Options(_) => {
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
            ApplicationScreen::ChangePage(ChangePageState{from, to}) => {
                let first = from;
                let second = to;
                #[cfg(debug_assertions)]
                graphics_context.clear_color(Color::new(255, 0, 0, 255));
                let slideshow = &self.slideshow.as_ref().unwrap();

                if let Some(transition) = &slideshow.transition {
                    let easing_amount = transition.easing_amount();
                    let forward_direction = second > first;
                    let sign = if forward_direction { 1.0 } else { -1.0 };

                    match transition.transition_type {
                        // These two transitions are almost identical... maybe I should refactor this later.
                        SlideTransitionType::HorizontalSlide => {
                            graphics_context.camera.y = 0.0;

                            graphics_context.camera.x = 0.0 - graphics_context.logical_width() as f32 * easing_amount * sign;
                            slideshow.try_to_draw_page(graphics_context, default_font, first as usize);

                            graphics_context.camera.x = (sign * graphics_context.logical_width() as f32) - (graphics_context.logical_width() as f32 * easing_amount) * sign;
                            slideshow.try_to_draw_page(graphics_context, default_font, second as usize);
                        },
                        SlideTransitionType::VerticalSlide => {
                            graphics_context.camera.x = 0.0;

                            graphics_context.camera.y = 0.0 - graphics_context.logical_height() as f32 * easing_amount * sign;
                            slideshow.try_to_draw_page(graphics_context, default_font, first as usize);

                            graphics_context.camera.y = (sign * graphics_context.logical_height() as f32) - (graphics_context.logical_height() as f32 * easing_amount) * sign;
                            slideshow.try_to_draw_page(graphics_context, default_font, second as usize);
                        },
                        SlideTransitionType::FadeTo(color) => {
                            // split time into two halves.
                            let half_max_time = transition.finish_time / 2.0;
                            let ease_function = transition.easing_function;
                            let fraction_of_completion = transition.finished_fraction();
                            let (alpha, page_to_draw) = {
                                // non-linear easing looks weird for this. For presumably an obvious reason.
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
                            slideshow.try_to_draw_page(graphics_context, default_font, page_to_draw as usize);
                            graphics_context.render_filled_rectangle(0.0, 0.0,
                                                                     graphics_context.logical_width() as f32,
                                                                     graphics_context.logical_height() as f32,
                                                                     color);
                        },
                    }
                }
            },
            ApplicationScreen::ShowingSlide(_) => {
                if let Some(slideshow) = &self.slideshow {
                    // graphics_context.camera.set_position(0.0, 0.0);
                    graphics_context.camera.x = 0.0;
                    graphics_context.camera.y = 0.0;
                    graphics_context.clear_color(Color::new(0, 0, 0, 255));
                    slideshow.try_to_draw_page(graphics_context, default_font, slideshow.current_page() as usize);
                }
            },
        }
    }

    pub fn handle_event(&mut self, graphics_context: &mut SDL2GraphicsContext, event_pump: &mut sdl2::EventPump, delta_time: f32) {
        match self.state.clone() {
            ApplicationScreen::Quit(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::SelectSlideToLoad(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::InvalidOrNoSlide(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::Options(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
            ApplicationScreen::ChangePage(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            }
            ApplicationScreen::ShowingSlide(state) => {
                state.handle_event(self, graphics_context, event_pump, delta_time);
            },
        }
    }
}
