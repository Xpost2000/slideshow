pub use crate::application_states::*;

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
        if let None = &app.slideshow {
            app.state = ApplicationScreen::InvalidOrNoSlide(InvalidOrNoSlideState);
        }
    }
}
