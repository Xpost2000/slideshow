use sdl2::event::Event as SDLEvent;
use sdl2::keyboard::Keycode as SDLKeycode;

pub use crate::utility::*;
pub use crate::slide::*;
pub use crate::color::*;
pub use crate::graphics_context::*;
pub use crate::application::*;

pub trait ApplicationScreenState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    }
}

// Unit structs for trait implementation.
#[derive(Clone)]
pub struct InvalidOrNoSlideState;
#[derive(Clone)]
pub struct OptionsState;
#[derive(Clone)]
pub struct ShowingSlideState;
#[derive(Clone)]
pub struct SelectSlideToLoadState;
#[derive(Clone)]
pub struct ChangePageState {pub from: isize, pub to: isize,}
#[derive(Clone)]
pub struct QuitState;
#[derive(Clone)]
pub enum ApplicationScreen {
    InvalidOrNoSlide(InvalidOrNoSlideState),
    Options(OptionsState),
    ShowingSlide(ShowingSlideState),
    SelectSlideToLoad(SelectSlideToLoadState),
    ChangePage(ChangePageState),
    Quit(QuitState),
}

impl ApplicationScreenState for QuitState {}

impl ApplicationScreenState for InvalidOrNoSlideState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::Options(OptionsState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::SelectSlideToLoad(SelectSlideToLoadState);
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");
        graphics_context.clear_color(Color::new(10, 10, 16, 255));
        graphics_context.logical_resolution = VirtualResolution::Display;
        graphics_context.render_text_justified(default_font,
                                               TextBounds::EntireScreen,
                                               TextJustification::center(),
                                               "Invalid / No slide file",
                                               graphics_context.font_size_percent(0.073),
                                               COLOR_WHITE,
                                               sdl2::ttf::FontStyle::NORMAL);
    }
}

impl ApplicationScreenState for OptionsState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        let resolutions = graphics_context.get_avaliable_resolutions();
        let resolution_count = resolutions.iter().count();
        app.currently_selected_resolution = clamp(app.currently_selected_resolution,
                                                   0, resolution_count - 1);
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::F), ..} => {
                    graphics_context.toggle_fullscreen();
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Return), .. } =>  {
                    let resolution_list = graphics_context.get_avaliable_resolutions();
                    if let Some(resolution_pair) = resolution_list.get(app.currently_selected_resolution) {
                        graphics_context.set_resolution((resolution_pair.0 as u32, resolution_pair.1 as u32));
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                    if app.currently_selected_resolution > 0 {
                        app.currently_selected_resolution -= 1;
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                    app.currently_selected_resolution += 1;
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::SelectSlideToLoad(SelectSlideToLoadState);
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");
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
            .skip(app.currently_selected_resolution)
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
    }
}

impl ApplicationScreenState for ChangePageState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown {..} => {
                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                    if let Some(slideshow) = &mut app.slideshow {
                        slideshow.finish_transition();
                    }
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");
        let first = self.from;
        let second = self.to;
        #[cfg(debug_assertions)]
        graphics_context.clear_color(Color::new(255, 0, 0, 255));
        let slideshow = &app.slideshow.as_ref().unwrap();

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
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
        let first = self.from;
        let second = self.to;
        if let Some(slideshow) = &mut app.slideshow {
            let valid_transition = slideshow.get(first as usize).is_some() && slideshow.get(second as usize).is_some();
            if let None = slideshow.transition {
                app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
            } else if let Some(transition) = &mut slideshow.transition {
                if valid_transition && !transition.finished_transition() {
                    transition.time += delta_time;
                } else {
                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                    transition.time = 0.0;
                }
            }
        }
    }
}

impl ApplicationScreenState for ShowingSlideState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Escape), .. } =>  {
                    graphics_context.clear_resources();
                    app.slideshow = None;
                    app.state = ApplicationScreen::InvalidOrNoSlide(InvalidOrNoSlideState);
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
                    if let Some(slideshow) = &mut app.slideshow {
                        graphics_context.clear_resources();
                        slideshow.reload().expect("should be successful");
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. } => {
                    if let Some(slideshow) = &mut app.slideshow {
                        app.state = ApplicationScreen::ChangePage(
                            ChangePageState{
                                from: slideshow.current_page(),
                                to: slideshow.next_page()
                            });
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                    if let Some(slideshow) = &mut app.slideshow {
                        app.state = ApplicationScreen::ChangePage(
                            ChangePageState{
                                from: slideshow.current_page(),
                                to: slideshow.previous_page()
                            });
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::SelectSlideToLoad(SelectSlideToLoadState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::Options(OptionsState);
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");
        if let Some(slideshow) = &app.slideshow {
            // graphics_context.camera.set_position(0.0, 0.0);
            graphics_context.camera.x = 0.0;
            graphics_context.camera.y = 0.0;
            graphics_context.clear_color(Color::new(0, 0, 0, 255));
            slideshow.try_to_draw_page(graphics_context, default_font, slideshow.current_page() as usize);
        }
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
    }
}

impl ApplicationScreenState for SelectSlideToLoadState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    delta_time: f32) {
        let directory_listing = std::fs::read_dir(&app.current_working_directory).expect("Failed to get directory listing?");
        app.currently_selected_directory = clamp(app.currently_selected_directory,
                                                  0, directory_listing.into_iter().count()-1);
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::Quit {..} => {
                    app.state = ApplicationScreen::Quit(QuitState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Return), .. } |
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Right), .. }  =>  {
                    let directory_listing = std::fs::read_dir(&app.current_working_directory).expect("Failed to get directory listing?");
                    let selected_path = directory_listing.into_iter().nth(app.currently_selected_directory);

                    if let Some(path) = selected_path {
                        let path = path.expect("bad permission?");
                        let file_type = path.file_type().unwrap();

                        if file_type.is_dir() {
                            app.current_working_directory = path.path();
                            app.currently_selected_directory = 0;
                        } else {
                            graphics_context.clear_resources();

                            let new_slide = Slide::new_from_file(path.path().to_str().expect("bad unicode"));
                            app.slideshow = new_slide;

                            app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                        }
                    }
                },

                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Left), .. } => {
                    app.current_working_directory.pop();
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Up), .. } => {
                    if app.currently_selected_directory > 0 {
                        app.currently_selected_directory -= 1;
                    }
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::Down), .. } => {
                    app.currently_selected_directory += 1;
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::O), .. } => {
                    app.state = ApplicationScreen::Options(OptionsState);
                },
                SDLEvent::KeyDown { keycode: Some(SDLKeycode::L), .. } => {
                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                },
                _ => {}
            }
        }
    }

    fn draw(&self,
            app: &ApplicationState,
            graphics_context: &mut SDL2GraphicsContext) {
        let default_font = graphics_context.add_font("data/fonts/libre-baskerville/LibreBaskerville-Regular.ttf");

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

        let directory_listing = std::fs::read_dir(&app.current_working_directory).expect("Failed to get directory listing?");

        graphics_context.render_text(default_font,
                                     ((graphics_context.logical_width() as i32 / 2) - (width as i32) / 2) as f32,
                                     heading_height as f32,
                                     #[cfg(target_os = "windows")] // Weird thing that looks like a drive root? ?//?DRIVE_LETTER:/
                                     &format!("{}", &app.current_working_directory.to_str().unwrap())[4..],
                                     #[cfg(target_os = "unix")]
                                     &format!("{}", &app.current_working_directory.to_str().unwrap()),
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
            skip(app.currently_selected_directory)
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
    }

    fn update(&self,
              app: &mut ApplicationState,
              delta_time: f32) {
        if let None = &app.slideshow {
            app.state = ApplicationScreen::InvalidOrNoSlide(InvalidOrNoSlideState);
        }
    }
}
