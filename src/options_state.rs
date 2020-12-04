pub use crate::application_states::*;

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
