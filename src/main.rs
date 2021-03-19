#![allow(clippy::collapsible_match)]
#![allow(clippy::single_match)]

use std::cell::RefCell;

mod frame_counter;
mod graphics;
mod shader_compilation;
mod vertex;

use graphics::GraphicsState;

fn main() {
    {
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
            // .level_for("gfx_backend_vulkan", log::LevelFilter::Warn)
            .level_for("gfx_backend_vulkan", log::LevelFilter::Off)
            .chain(std::io::stdout())
            .apply()
            .unwrap();
    } // fern::Dispatch::new()

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();

    let (imgui_context, imgui_platform) = {
        let context = Box::leak(Box::new(RefCell::new(imgui::Context::create())));
        let platform = Box::leak(Box::new(RefCell::new(
            imgui_winit_support::WinitPlatform::init(&mut *context.borrow_mut()),
        )));
        platform.borrow_mut().attach_window(
            context.borrow_mut().io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        (context, platform)
    };

    let mut graphics_state = GraphicsState::new(window, imgui_context);

    let mut view = graphics::SfView {
        center: (500.0, 500.0),
        size: (1000.0, 1000.0),
        rotation: 0.0,
    };
    let mut renderer_with_view = graphics::renderers::RendererWithView::new(&mut graphics_state);
    let mut renderer_simple_triangle =
        graphics::renderers::RendererSimpleTriangle::new(&mut graphics_state);
    let mut renderer_glyph = graphics::renderers::RendererGlyph::new(&graphics_state);
    let mut renderer_imgui =
        graphics::renderers::RendererImgui::new(&graphics_state, imgui_context, imgui_platform);

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
                            virtual_keycode: Some(keycode),
                            ..
                        } => {
                            //
                            match keycode {
                                VirtualKeyCode::LShift => {
                                    view.size.0 += 10.0;
                                    view.size.1 += 10.0;
                                }
                                VirtualKeyCode::LControl => {
                                    view.size.0 -= 10.0;
                                    view.size.1 -= 10.0;
                                }
                                VirtualKeyCode::E => {
                                    view.rotation -= 5.0;
                                }
                                VirtualKeyCode::Q => {
                                    view.rotation += 5.0;
                                }
                                VirtualKeyCode::W | VirtualKeyCode::Up => {
                                    view.center.1 += 5.0;
                                }
                                VirtualKeyCode::A | VirtualKeyCode::Left => {
                                    view.center.0 += 5.0;
                                }
                                VirtualKeyCode::S | VirtualKeyCode::Down => {
                                    view.center.1 -= 5.0;
                                }
                                VirtualKeyCode::D | VirtualKeyCode::Right => {
                                    view.center.0 -= 5.0;
                                }
                                _ => {}
                            }
                        }
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
            Event::RedrawRequested(_) => match graphics_state.begin_current_frame() {
                Err(wgpu::SwapChainError::Lost) => graphics_state.resize(),
                Err(wgpu::SwapChainError::OutOfMemory) => {
                    *control_flow = winit::event_loop::ControlFlow::Exit
                }
                Err(e) => log::warn!("{:?}", e),

                Ok(mut current_frame) => {
                    renderer_with_view.draw(&mut current_frame, &view);
                    renderer_simple_triangle.draw(&mut current_frame);
                    renderer_glyph.draw(&mut current_frame);
                    renderer_imgui.draw(&mut current_frame);
                    current_frame.finish_and_present();
                }
            },
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

        // graphics_state.input(&event);
        renderer_imgui.imgui_platform.borrow_mut().handle_event(
            renderer_imgui.imgui_context.borrow_mut().io_mut(),
            &graphics_state.window,
            &event,
        );
    });
}
