use anyhow::Error;

use renderer::{RenderableState, Renderer};

pub struct ErrScreenState {
    error: Option<anyhow::Error>
}

impl ErrScreenState {
    pub fn set_error(&mut self, error: anyhow::Error) {
        self.error = Some(error);
    }
}

impl RenderableState for ErrScreenState {
    fn get_state_id(&self) -> u32 { 1 }

    fn is_cursor_visible(&self) -> bool { true }

    fn new(_renderer: &mut Renderer, _window: &winit::window::Window) -> anyhow::Result<Box<Self>, anyhow::Error> where Self: Sized {
        Ok(Box::new(Self { error: None }))
    }

    fn update(&mut self, _renderer: &Renderer, _delta_time: f32) -> Result<(), Error> {
        Ok(())
    }

    fn on_resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) -> anyhow::Result<(), anyhow::Error> {
        Ok(())
    }

    fn handle_keys(&mut self, _input_manager: &renderer::input_manager::InputManager)  -> anyhow::Result<bool, anyhow::Error> {
        Ok(true)
    }

    fn handle_mouse(&mut self, _mouse_dx: f64, _mouse_dy: f64) -> anyhow::Result<bool, anyhow::Error> {
        Ok(true)
    }

    fn render<'a>(&'a mut self, renderer: &'a Renderer, _render_pass: &mut wgpu::RenderPass<'a>, _delta_time: f32) -> anyhow::Result<(), anyhow::Error> {
        
        let text = if let Some(err) = &self.error {
            format!("{}", err)
        } else {
            format!("An unknown error occurred!")
        };

        renderer.queue_string(&text, (5.0, 150.0), [1.0, 1.0, 1.0, 1.0], 25.0);

        Ok(())
    }

    fn exit(&mut self) -> anyhow::Result<(), anyhow::Error> {
        Ok(())
    }
}