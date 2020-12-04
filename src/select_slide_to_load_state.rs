pub use crate::application_states::*;

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
        let heading_font_size = graphics_context.font_size_percent(0.04);
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

        let mut draw_cursor_y = (heading_height as f32 * 2.5);
        let listings_to_show = 13;
        // TODO: refactor
        graphics_context.render_filled_rectangle((((graphics_context.logical_width() as i32 / 2)) - 250) as f32,
                                                 draw_cursor_y-10.0,
                                                 (graphics_context.logical_width()/2) as f32,
                                                 listings_to_show as f32 * graphics_context.font_size_percent(0.056) as f32,
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
                        0.051
                    } else {
                        0.042
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
    }
}
