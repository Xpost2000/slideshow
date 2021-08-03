pub use crate::application_states::*;

impl ApplicationScreenState for InvalidOrNoSlideState {
    fn handle_event(&self,
                    app: &mut ApplicationState,
                    graphics_context: &mut SDL2GraphicsContext,
                    event_pump: &mut sdl2::EventPump,
                    _delta_time: f32) {
        for event in event_pump.poll_iter() {
            match event {
                SDLEvent::DropFile { filename, .. } => {
                    graphics_context.clear_resources();

                    let new_slide = Slide::new_from_file(&filename);
                    app.slideshow = new_slide;

                    app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
                },
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
                _ => {
                    #[cfg(debug_assertions)]
                    {
                        println!("Unhandled event type: {:#?}", event);
                    }
                }
            }
        }
    }

    fn draw(&self,
            _app: &ApplicationState,
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
