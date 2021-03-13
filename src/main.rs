use crate::graphics_state::GraphicsState;

mod frame_counter;
mod graphics_state;
mod shader_compilation;
mod vertex;

fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] [{}] [{}] : {}",
                chrono::Local::now().format("%H:%M:%S.%3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .level_for("naga", log::LevelFilter::Error)
        .level_for("gfx_backend_vulkan", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let mut graphics_state = GraphicsState::new(window);

    event_loop.run(move |event, _, control_flow| {
        use winit::event::*;

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == graphics_state.window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = winit::event_loop::ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(..) => {
                        graphics_state.resize();
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        // new_inner_size is &&mut so w have to dereference it twice
                        graphics_state.resize();
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                graphics_state.update();
                match graphics_state.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => graphics_state.resize(),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => {
                        *control_flow = winit::event_loop::ControlFlow::Exit
                    }
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => log::warn!("{:?}", e),
                }
            }
            Event::MainEventsCleared => {
                // incoming networking here
                // updating + physics here
                // outgoing networking again here?
                // draw:
                graphics_state.window.request_redraw();
                // std::thread::sleep(std::time::Duration::from_micros(1));
            }
            _ => {}
        }

        graphics_state.input(&event);
    });
}
