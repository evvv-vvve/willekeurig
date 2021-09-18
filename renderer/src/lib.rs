// modules

pub mod texture;
pub mod camera_uniform;
pub mod camera;
pub mod vertex;
pub mod input_manager;

// imports
use std::{cell::RefCell, sync::{Arc, RwLock}};

use anyhow::{Error, Result, anyhow};
use input_manager::InputManager;

use wgpu_glyph::{GlyphBrush, Section, Text, ab_glyph};
use winit::{event::{DeviceEvent, KeyboardInput}, window::Window};

//use texture;

pub enum RenderingError {
    SurfaceError(wgpu::SurfaceError),
    GenericError(Error)
}

pub trait RenderableState {
    fn get_state_id(&self) -> u32;

    fn is_cursor_visible(&self) -> bool;

    fn new(renderer: &mut Renderer, window: &Window) -> Result<Box<Self>, Error> where Self: Sized;

    fn update(&mut self, renderer: &Renderer, delta_time: f32) -> Result<(), Error>;

    fn on_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> Result<(), Error>;
    fn handle_keys(&mut self, input_manager: &InputManager)  -> Result<bool, Error>;
    fn handle_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) -> Result<bool, Error>;

    fn render<'a>(&'a mut self, renderer: &'a Renderer, render_pass: &mut wgpu::RenderPass<'a>, delta_time: f32) -> Result<(), Error>;

    fn exit(&mut self) -> Result<(), Error>;
}

pub struct Renderer {
    adapter: wgpu::Adapter,
    surface: wgpu::Surface,
    device: Arc<RwLock<wgpu::Device>>,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    depth_texture: texture::Texture,

    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
    local_spawner: futures::executor::LocalSpawner,
    glyph_brush: RefCell<GlyphBrush<()>>,

    states: Vec<RefCell<Box<dyn RenderableState>>>,
    cursor_visible: bool,

    input_manager: InputManager,
} 

// getters
impl Renderer {
    pub fn get_device(&self) -> Arc<RwLock<wgpu::Device>> { self.device.clone() }

    pub fn get_size(&self) -> winit::dpi::PhysicalSize<u32> { self.size }

    pub fn get_adapter(&self) -> &wgpu::Adapter { &self.adapter }
    pub fn get_queue(&self) -> &wgpu::Queue { &self.queue }
    pub fn get_surface_config(&self) -> &wgpu::SurfaceConfiguration { &self.surface_config }
}

impl Renderer {
    pub fn peek(&self) -> Option<&RefCell<Box<dyn RenderableState>>> {
        let len = self.states.len();

        if len > 0 {
            self.states.get(len - 1)
        } else {
            None
        }
    }

    pub fn push_state(&mut self, window: &Window, render_state: Box<dyn RenderableState>) -> Result<bool, Error> {
        for state in &self.states {
            if state.borrow().get_state_id() == render_state.get_state_id() {
                return Ok(false);
            }
        }

        self.states.push(RefCell::new(render_state));
        
        match self.update_cursor_visibility(window) {
            Ok(_) => { 
                eprintln!("[LOG] Pushed new state");
                Ok(true)
            },
            Err(err) => {
                Err(anyhow!(err))
            }
        }
    }

    pub fn pop_state(&mut self) -> bool {
        if let Some(_) = self.states.pop() {
            true
        } else {
            false
        }
    }

    pub fn clear_states(&mut self) {
        while let Some(_) = self.states.pop() { }
    }
}

impl Renderer {
    pub async fn new(window: &Window) -> Renderer {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Primary: Vulkan + Metal + DX12 + Browser WebGPU, wgpu first tier support
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface)
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: Some("Device Descriptor")
            },
            None
        ).await.unwrap();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo
        };

        

        surface.configure(&device, &surface_config);

        let depth_texture = texture::Texture::create_depth_texture(&device, &surface_config, "depth texture");

        // Font stuff, worry about making it work later
        let staging_belt = wgpu::util::StagingBelt::new(1024);
        let local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!(
            "../../res/Inconsolata-Regular.ttf"
        )).unwrap();

        let glyph_brush = wgpu_glyph::GlyphBrushBuilder::using_font(inconsolata)
            .build(&device, surface.get_preferred_format(&adapter).unwrap());

        Self {
            adapter,
            surface,
            device: Arc::new(RwLock::new(device)),
            queue,
            surface_config,
            size,

            staging_belt,
            local_pool,
            local_spawner,
            glyph_brush: RefCell::new(glyph_brush),

            depth_texture,

            states: Vec::new(),
            cursor_visible: true,

            input_manager: InputManager::new(),
        }
    }

    pub fn update_cursor_visibility(&mut self, window: &Window) -> Result<(), Error> {
        if let Some(state) = self.peek() {
            let visible = state.borrow().is_cursor_visible();

            self.cursor_visible = visible;

            match window.set_cursor_grab(!self.cursor_visible) {
                Ok(_) => {
                    window.set_cursor_visible(self.cursor_visible);
                    Ok(())
                },
                Err(err) => {
                    Err(anyhow!(err))
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> Result<(), Error> {
        self.size = new_size;
        self.surface_config.width = new_size.width;
        self.surface_config.height = new_size.height;

        self.surface.configure(&self.device.clone().read().unwrap(), &self.surface_config);
        
        self.depth_texture = texture::Texture::create_depth_texture(
            &self.device.clone().read().unwrap(), &self.surface_config, "depth texture"
        );

        let states_len = self.states.len();

        if states_len > 0 {
            if let Some(state) = self.peek() {
                match state.borrow_mut().on_resize(new_size) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(anyhow!(err))
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn input(&mut self, window: &Window, event: &DeviceEvent, focused: bool) -> Result<bool, Error> {
        if !focused {
            Ok(false)
        } else {
            let res = match event {
                DeviceEvent::Key(
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    }
                ) => {
                    self.input_manager.process_keys(*key, *state);

                    if let Some(state_) = self.peek() {
                        match state_.borrow_mut().handle_keys(&self.input_manager) {
                            Ok(resp) => resp,
                            Err(err) => return Err(anyhow!(err))
                        }
                    } else {
                        false
                    }
                },
                DeviceEvent::MouseMotion { delta: (delta_x, delta_y) } => {
                    if let Some(state_) = self.peek() {
                        match state_.borrow_mut().handle_mouse(*delta_x, *delta_y) {
                            Ok(resp) => resp,
                            Err(err) => return Err(anyhow!(err))
                        }
                    } else {
                        true
                    }
                }
                _ => false,
            };

            match self.peek() {
                Some(state_) => {
                    if state_.borrow().is_cursor_visible() != self.cursor_visible {
                        match self.update_cursor_visibility(window) {
                            Ok(_) => { },
                            Err(err) => return Err(anyhow!(err))
                        }
                    }
                },
                None => { }
            }

            self.input_manager.clear_just_pressed();
            self.input_manager.clear_just_released();

            Ok(res)
        }
    }

    pub fn update(&mut self, delta_time: f32) -> Result<(), Error> {
        if let Some(state) = self.peek() {
            state.borrow_mut().update(&self, delta_time)
        } else {
            Ok(())
        }
    }

    pub fn render(&mut self, delta_time: f32) -> Result<(), RenderingError> {
        let frame = match self.surface.get_current_frame() {
            Ok(surface_frame) => surface_frame.output,
            Err(surface_err) => return Err(RenderingError::SurfaceError(surface_err))
        };
                
        let mut encoder = self.device.clone()
            .read().unwrap()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") }
        );

        {
            let depth_texture = &self.depth_texture.view;
            
            // render_pass needs some values stored in borrowed_state.
            // at the end of the scope, values are dropped in the opposite
            // order they're created in (last created is dropped first),
            // so we create borrowed_state before render_pass so that
            // the values on borrowed_state aren't destroyed while
            // render_pass is referencing them :)
            let mut borrowed_state;

            let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &frame_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0
                            }),
                            store: true
                        }
                    }
                ],
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_texture,
                        depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true
                        }),
                        stencil_ops: None
                    }
                )
            });

            let peek = self.peek();
            if let Some(state) = peek {
                borrowed_state = state.borrow_mut();

                match borrowed_state.render(&self, &mut render_pass, delta_time) {
                    Ok(_) => { },
                    Err(err) => return Err(RenderingError::GenericError(err))
                }
            }
        }

        self.glyph_brush.borrow_mut()
            .draw_queued(
                &self.device.clone().read().unwrap(),
                &mut self.staging_belt,
                &mut encoder,
                &frame.texture.create_view(&wgpu::TextureViewDescriptor::default()),
                self.size.width,
                self.size.height,
            )
            .expect("Draw queued");

        // Submit the work!
        self.staging_belt.finish();
        self.queue.submit(std::iter::once(encoder.finish()));

        // Recall unused staging buffers
        use futures::task::SpawnExt;

        self.local_spawner
            .spawn(self.staging_belt.recall())
            .expect("Recall staging belt");

        self.local_pool.run_until_stalled();

        Ok(())
    }

    pub fn queue_string(&self, text: &str, pos: (f32, f32), col: [f32; 4], scale: f32) {
        self.glyph_brush.borrow_mut().queue(Section {
            screen_position: pos,
            bounds: (self.size.width as f32, self.size.height as f32),
            text: vec![
                Text::new(text)
                    .with_color(col)
                    .with_scale(scale)
                ],
            ..Section::default()
        });
    }

    pub fn exit(&mut self) -> Result<(), Error> {
        if let Some(state) = self.peek() {
            state.borrow_mut().exit()
        } else {
            Ok(())
        }
    }
}