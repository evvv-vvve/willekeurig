pub mod states;
pub mod camera_controller;
pub mod player;

use std::time::Instant;

use anyhow::{Result, Error, anyhow};
use futures::executor::block_on;
use winit::{event::*, event_loop::{EventLoop, ControlFlow}, window::{Window, WindowBuilder}};

use renderer::{RenderableState, Renderer, RenderingError};


use states::{willekeuirig_state::WillekeuirigState, err_screen_state::ErrScreenState};

enum RunError {
    FatalError(Error),
    UpdateError(Error),
    UnhandledError(Error)
}

fn main() {
    //env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    
    window.set_min_inner_size(Some(winit::dpi::PhysicalSize { width: 800, height: 600 }));
    window.set_title("Willekeurig");

    let mut renderer = block_on(Renderer::new(&window));

    match WillekeuirigState::new(&mut renderer, &window) {
        Ok(state) => {
            match renderer.push_state(&window, state) {
                Ok(_) => { },
                Err(err) => {
                    eprintln!("An error occurred while pushing game state: {}", err);
                    std::process::exit(1);
                }
            }
        },
        Err(err) => {
            /* TODO: Error handling */
            eprintln!("An error occurred while creating game state: {}", err);
            std::process::exit(1);
        }
    }
    
    //let mut state = block_on(RenderState::new(&window)).unwrap();
    let mut last_render_time = std::time::Instant::now();
    let mut focused = false;

    //let (sender, receiver) = futures::channel::oneshot::channel::<(Chunk, Vector3<i32>, usize)>();
    
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match run_loop(event, control_flow, &window, &mut renderer, &mut focused, &mut last_render_time) {
            Ok(()) => { },
            Err(run_error) => {
                match run_error {
                    RunError::FatalError(err) => {
                        eprintln!("[FATAL ERROR] {}", err);
                        std::process::exit(1);
                    },
                    RunError::UnhandledError(err) => { eprintln!("[UNHANDLED ERROR] {}", err); },
                    RunError::UpdateError(err) => {
                        match ErrScreenState::new(&mut renderer, &window) {
                            Ok(mut state) => {
                                state.set_error(err);
                                match renderer.push_state(&window, state) {
                                    Ok(_) => { },
                                    Err(err) => {
                                        eprintln!("An error occurred while trying to display error screen: {}", err);
                                        std::process::exit(1);
                                    }
                                }
                            },
                            Err(err) => {
                                eprintln!("An error occurred while creating error screen: {}", err);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            }
        }
    });
}

fn run_loop(event: Event<()>, control_flow: &mut ControlFlow, window: &Window, renderer: &mut Renderer, focused: &mut bool,
   last_render_time: &mut Instant) -> Result<(), RunError> {
    // this should only ever be an unhandled error, since (theoretically) they shouldn't cause
    // any major problems, and the application should run normally
    let mut unhandled_error: Option<RunError> = None;
    
    match event {
        Event::WindowEvent { ref event, window_id }
         if window_id == window.id() => {
            match event {
                WindowEvent::Focused(is_focused) => *focused = *is_focused,
                WindowEvent::CloseRequested => {
                    match renderer.exit() {
                        Ok(_) => *control_flow = ControlFlow::Exit,
                        Err(err) => {
                            return Err(RunError::FatalError(anyhow!(format!("An error occurred while exiting the application: {}", err))))
                        }
                    }
                },
                WindowEvent::Resized(physical_size) => {
                    match renderer.resize(*physical_size) {
                        Ok(_) => { },
                        Err(err) => {
                            return Err(RunError::FatalError(anyhow!(format!("An error occurred while resizing the application window: {}", err))))
                        }
                    }
                },
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    match renderer.resize(**new_inner_size) {
                        Ok(_) => { },
                        Err(err) => {
                            return Err(RunError::FatalError(anyhow!(format!("An error occurred while resizing the application window: {}", err))))
                        }
                    }
                },
                _ => {}
            }
        },
        Event::DeviceEvent {
            ref event,
            .. // We're not using device_id currently
        } => { 
            match renderer.input(&window, event, *focused) {
                Ok(_) => { },
                Err(err) => {
                    return Err(RunError::UpdateError(anyhow!(format!("An error occurred while updating input for the current state: {}", err))))
                }
            }
         }
        Event::RedrawRequested(_) => {
            let now = std::time::Instant::now();
            let dt = now - *last_render_time;

            match renderer.update(dt.as_secs_f32()) {
                Ok(_) => { },
                Err(err) => {
                    return Err(RunError::UpdateError(anyhow!(format!("An error occurred while updating the current state: {}", err))))
                }
            }

            match renderer.render(dt.as_secs_f32()) {
                Ok(_) => {},
                Err(render_error) => {
                    match render_error {
                        RenderingError::SurfaceError(surface_error) => {
                            match surface_error {
                                // Recreate the swap chain if lost
                                wgpu::SurfaceError::Lost => {
                                    eprintln!("[LOG] Lost swap chain! Recreating");
                                    
                                    match renderer.resize(renderer.get_size()) {
                                        Ok(_) => { },
                                        Err(err) => {
                                            return Err(RunError::FatalError(anyhow!(format!("An error occurred while resizing the application window: {}", err))))
                                        }
                                    }
                                },
                                // System ran out of memory! We should quit
                                // TODO: improve this later?
                                wgpu::SurfaceError::OutOfMemory => {
                                    *control_flow = ControlFlow::Exit;
                                    return Err(RunError::FatalError(anyhow!("Quitting because the system ran out of memory")))
                                },
                                // Other errors should be resolved by the next frame
                                err => unhandled_error = Some(RunError::UnhandledError(anyhow!(err)))
                            }
                        },
                        RenderingError::GenericError(err) => {
                            unhandled_error = Some(RunError::UnhandledError(err))
                        }
                    }
                }
            }

            *last_render_time = now;
        },
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once,
            // unless we manually request it
            window.request_redraw();
        },
        _ => {}
    }

    if let Some(err) = unhandled_error {
        Err(err)
    } else {
        Ok(())
    }
}