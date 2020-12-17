pub use crate::application_states::*;

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

        if let Some(transition) = &slideshow.get_current_page().unwrap().transition {
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
            if let None = slideshow.get_current_page().unwrap().transition {
                app.state = ApplicationScreen::ShowingSlide(ShowingSlideState);
            } else if let Some(transition) = &mut slideshow.get_current_page_mut().unwrap().transition {
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
