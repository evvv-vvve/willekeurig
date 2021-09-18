use anyhow::{Result, Error, anyhow};
use cgmath::Vector3;
use winit::{event::VirtualKeyCode, window::Window};

use rand::Rng;
use wgpu::util::DeviceExt;

use renderer::{Renderer, RenderableState, camera, texture, camera_uniform, vertex::Vertex};
use common::{block::Block, identifier::Identifier, registry::Registry};
use world;

use crate::player;

pub struct WillekeuirigState {
    //registry: Arc<Registry>,

    render_pipeline: wgpu::RenderPipeline,

    block_texture: texture::Texture,
    
    world: world::World,
    
    //camera: camera::Camera,
    player: player::Player,
    projection: camera::Projection,
    //camera_controller: camera_controller::CameraController,
    
    camera_uniform: camera_uniform::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    cursor_visible: bool,
}

impl RenderableState for WillekeuirigState {
    fn get_state_id(&self) -> u32 { 0 }

    fn is_cursor_visible(&self) -> bool { self.cursor_visible }

    fn new(renderer: &mut Renderer, window: &Window) -> Result<Box<Self>, Error> {
        let dev_clone = renderer.get_device().clone();
        let device = dev_clone.read().unwrap();

        let seed = rand::thread_rng().gen::<u32>();

        let texture_bytes = include_bytes!("../../../res/textures/block_atlas.png");
        let block_texture = texture::Texture::from_bytes(&device, &renderer.get_queue(),
            texture_bytes, "block_texture").unwrap();

        register_blocks()?;

        let world = world::World::new(seed, 5);

        let player_position = Vector3::unit_y() * 64.0;

        let player = player::Player::new(
            player_position,
            //Vector3::zero(), // rotation
            0.4,
            4.0,
        );

        let projection = camera::Projection::new(
            renderer.get_surface_config().width,
            renderer.get_surface_config().height,
            cgmath::Deg(45.0),
            0.1,
            1000.0
        );
        //let camera_controller = camera_controller::CameraController::new(4.0, 0.4);
        let mut camera_uniform = camera_uniform::CameraUniform::new();

        camera_uniform.update_view_proj(player.get_camera(), &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("uniform_bind_group_layout"),
            }
        );

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("uniform_bind_group"),
        });


        let shader_src = include_str!("../../../res/shaders/shader.wgsl");

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into())
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &block_texture.bind_group_layout,
                &camera_bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "main",
                buffers: &[ Vertex::desc()/*, instance::InstanceRaw::desc()*/ ]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: renderer.get_surface_config().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                }]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                clamp_depth: false,
                conservative: false
            },
            depth_stencil: Some(
                wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }
            ),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            }
        });

        match renderer.update_cursor_visibility(window) {
            Ok(_) => {
                Ok(Box::new(Self {
                    //registry: Arc::new(registry),
                    render_pipeline,

                    block_texture,
        
                    world,
                    
                    //camera,
                    //camera_controller,
                    player,
                    projection,
        
                    camera_uniform,
                    camera_buffer,
                    camera_bind_group,
        
                    cursor_visible: false, 
                }))
            },
            Err(err) => Err(anyhow!(err))
        }
    }

    fn on_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> Result<(), Error> {
        self.projection.resize(new_size.width, new_size.height);

        Ok(())
    }

    fn handle_keys(&mut self, input_manager: &renderer::input_manager::InputManager)  -> Result<bool, Error> {
        if input_manager.key_just_pressed(VirtualKeyCode::Escape) {
            self.cursor_visible = !self.cursor_visible;
        }

        Ok(
            if self.cursor_visible {
                false
            } else {
                self.player.process_keyboard(input_manager)
            }
        )
    }

    fn handle_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) -> Result<bool, Error> {
        if !self.cursor_visible {
            self.player.get_camera_controller_mut().process_mouse(mouse_dx, mouse_dy);
        }
        
        Ok(true)
    }

    fn update(&mut self, renderer: &Renderer, delta_time: f32) -> Result<(), Error> {
        self.world.create_or_destroy_chunks(&self.player.get_camera().pos_as_vec3());
        self.world.update(renderer.get_device().clone(), self.player.get_camera().pos_as_vec3());
        
        self.player.update(&mut self.world, delta_time);
        self.camera_uniform.update_view_proj(&self.player.get_camera(), &self.projection);
        renderer.get_queue().write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));

        Ok(())
    }

    fn render<'a>(&'a mut self, renderer: &'a Renderer, render_pass: &mut wgpu::RenderPass<'a>,
       delta_time: f32) -> Result<(), Error> {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.block_texture.bind_group, &[]);
        render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

        for (_, chunk) in self.world.get_renderable_chunks() {
            if let Some((vertex_buffer, index_buffer, indicies)) = chunk.get_buffers() {
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                //render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                                                
                render_pass.draw_indexed(0..indicies, 0, 0..1);
            }
        }

        self.render_text(renderer, delta_time);

        Ok(())
    }

    fn exit(&mut self) -> Result<(), Error> {
        /*if self.world.get_chunks_loading() > 0 {
            println!("Waiting for {} background threads to close...", self.world.get_chunks_loading());

            while self.world.get_chunks_loading() > 0 { self.world.fetch_chunks(); }
        }*/

        Ok(())
    }
}

impl WillekeuirigState {
    fn render_text<'a>(&'a self, renderer: &'a Renderer, delta_time: f32) {
        let backend_str = match &renderer.get_adapter().get_info().backend {
            wgpu::Backend::BrowserWebGpu => "BrowserWebGpu",
            wgpu::Backend::Dx11 => "Direct3D 11",
            wgpu::Backend::Dx12 => "Direct3D 12",
            wgpu::Backend::Gl => "OpenGL ES-3",
            wgpu::Backend::Metal => "Metal",
            wgpu::Backend::Vulkan => "Vulkan",
            wgpu::Backend::Empty => "Empty (Unknown/Testing)",
        };

        renderer.queue_string(
            &format!("Graphics adapter: {} ({})", &renderer.get_adapter().get_info().name, backend_str),
            (5.0, 5.0), [1.0, 1.0, 1.0, 1.0], 20.0
        );

        renderer.queue_string(
            &format!("X/Y/Z: {:.2} / {:.2} / {:.2}",
                self.player.get_camera().position.x, self.player.get_camera().position.y - 2.0, self.player.get_camera().position.z),
            (5.0, 30.0), [1.0, 1.0, 1.0, 1.0], 20.0
        );

        renderer.queue_string(
            &format!("FPS: {:.2}", 1.0 / delta_time),
            (5.0, 55.0), [1.0, 1.0, 1.0, 1.0], 20.0
        );

        renderer.queue_string(
            &format!("Rotation (Yaw/Pitch): {:.3} / {:.3} (Facing {})",
                cgmath::Deg::from(self.player.get_camera().yaw).0,
                cgmath::Deg::from(self.player.get_camera().pitch).0,
                self.player.get_camera_controller().get_facing_dir(&self.player.get_camera()).as_string()
            ), (5.0, 80.0), [1.0, 1.0, 1.0, 1.0], 20.0
        );

        renderer.queue_string(
            &format!("Chunks generating: {} (max {}, {} in queue)",
                0, 0, 0//self.world.get_chunks_loading(), self.world.get_max_chunks(), self.world.get_chunk_queue_count()
            ), (5.0, 105.0), [1.0, 1.0, 1.0, 1.0], 20.0
        );

        renderer.queue_string(
            &format!("Player flying: {}", self.player.is_flying()),
            (5.0, 130.0), [1.0, 1.0, 1.0, 1.0], 20.0
        );

        let p_pos = Vector3::new(
            self.player.get_camera().position.x,
            self.player.get_camera().position.y,
            self.player.get_camera().position.z,
        );

        let chunk_pos = match self.world.get_chunk_from_world(&p_pos) {
            Some(chunk) => {
                let chunk_pos = chunk.get_pos();
                format!("[{},{},{}]", chunk_pos.x, chunk_pos.y, chunk_pos.z)
            },
            None => format!("[??,??,??]")
        };

        renderer.queue_string(
            &format!("Chunk: {}", chunk_pos),
            (5.0, 155.0), [1.0, 1.0, 1.0, 1.0], 20.0
        );
    }
}

fn register_blocks() -> Result<(), Error> {
    let mut grass_block = Block::new(Identifier::from_str("willekeurig:grass_block")?,0.0, 0.0);
    grass_block.set_texture_bottom(64.0, 0.0); 
    grass_block.set_side_textures(32.0, 0.0);

    let dirt = Block::new(Identifier::from_str("willekeurig:dirt")?, 64.0, 0.0);
    let stone = Block::new(Identifier::from_str("willekeurig:stone")?, 0.0, 32.0);

    let mut registry = Registry::new();

    registry.register_block(grass_block)?;

    registry.register_block(dirt)?;

    registry.register_block(stone)?;

    registry.make_current();

    Ok(())
}