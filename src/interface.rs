use egui_glium::EguiGlium;
use glium::{glutin, Display, Surface};
use glutin::event::Event;
use glutin::event_loop::{ControlFlow, EventLoop};

use crate::window_manager::WindowManager;

use crate::widgets::Widgets;

use egui::{Align, Color32, Layout, Visuals};

use std::time::{Duration, Instant};

const FPS: u64 = 60;

pub struct Interface {
    display: Display,
    egui_glium: EguiGlium,
    window_manager: WindowManager,
    widgets: Widgets,
}

impl Interface {
    pub fn new(event_loop: &EventLoop<()>, widgets: Widgets) -> Self {
        let window_builder = glutin::window::WindowBuilder::new()
            .with_resizable(false)
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top(true);

        let context_builder = glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true);

        let display = glium::Display::new(window_builder, context_builder, event_loop).unwrap();
        let egui_glium = EguiGlium::new(&display);

        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(Color32::LIGHT_GRAY);
        egui_glium.ctx().set_visuals(visuals);

        let window_manager = WindowManager::new("INSIDE", &display).unwrap();

        Self {
            display,
            egui_glium,
            window_manager,
            widgets,
        }
    }

    fn redraw(&mut self, control_flow: &mut ControlFlow) {
        self.egui_glium.begin_frame(&self.display);

        let mut open = true;
        egui::Window::new("Inside Hacks")
            .resizable(false)
            .fixed_pos((0.0, 0.0))
            .fixed_size((300.0, 50.0))
            .open(&mut open)
            .show(self.egui_glium.ctx(), |ui| {
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    self.widgets.display(ui);
                });
                self.window_manager
                    .update_window_size(&self.display, ui.ctx().used_rect());
            });

        let (needs_repaint, shapes) = self.egui_glium.end_frame(&self.display);

        if !open {
            self.widgets.close();
            *control_flow = ControlFlow::Exit;
        } else if needs_repaint {
            self.display.gl_window().window().request_redraw();
            *control_flow = ControlFlow::Poll;
        } else {
            *control_flow = ControlFlow::Poll;
        }

        let mut target = self.display.draw();

        let color = egui::Rgba::TRANSPARENT;
        target.clear_color_srgb(color[0], color[1], color[2], color[3]);

        self.egui_glium.paint(&self.display, &mut target, shapes);

        target.finish().unwrap();
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            let start_time = Instant::now();
            match event {
                Event::WindowEvent { event, .. } => {
                    self.egui_glium.on_event(&event);
                    if self.egui_glium.is_quit_event(&event) {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::MainEventsCleared => {
                    self.redraw(control_flow);
                    self.window_manager.update_window_pos(&self.display);
                    self.window_manager.update_window_visibility();

                    let elapsed = Instant::now().duration_since(start_time).as_millis() as u64;

                    let wait_time = match 1000 / FPS >= elapsed {
                        true => 1000 / FPS - elapsed,
                        false => 0,
                    };
                    let new_inst = start_time + Duration::from_millis(wait_time);

                    if *control_flow != ControlFlow::Exit {
                        *control_flow = ControlFlow::WaitUntil(new_inst);
                    }
                }
                _ => (),
            };
        });
    }
}
