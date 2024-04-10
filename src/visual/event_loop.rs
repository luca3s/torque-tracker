use winit::{
    event::{Event, Modifiers, WindowEvent},
    keyboard::{Key, ModifiersState, NamedKey},
    window::Window,
};

use super::{
    draw_buffer::DrawBuffer,
    gpu::GPUState,
    ui::{
        header::Header,
        pages::page::{AllPages, Page, PagesEnum},
    },
};

pub fn run() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let window = Window::new(&event_loop).unwrap();

    let mut gpu_state = pollster::block_on(GPUState::new(&window));

    let mut draw_buffer = DrawBuffer::new();
    let mut modifiers = Modifiers::default();
    let mut ui_pages = AllPages::new();

    let ui_header = Header {};
    ui_header.draw_constant(&mut draw_buffer);

    let window = &window;
    let _ = event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            window_id: _, // can ignore because i only use one window
            ref event,
        } => match event {
            // as soon as pop-up windows are working here should be a pop-up check, before exiting
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(pyhsical_size) => {
                gpu_state.resize(*pyhsical_size);
                // on macos redraw needs to be requested after resizing
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _,
            } => {
                // window_state.resize(**new_inner_size);
                // due to a version bump in winit i dont know anymore how to handle this event so i just ignore it for know and see if it makes problems in the future
                println!("Window Scale Factor Changed");
            }
            WindowEvent::RedrawRequested => {
                // draw the new frame buffer
                ui_pages.draw(&mut draw_buffer);
                println!("redraw");
                // notify the windowing system that drawing is done and the new buffer is about to be pushed
                window.pre_present_notify();
                // push the framebuffer into GPU and render it onto the screen
                match gpu_state.render(draw_buffer.framebuffer.flatten()) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => gpu_state.resize(gpu_state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                    Err(e) => eprint!("{:?}", e),
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => {
                if !is_synthetic {
                    if event.logical_key == Key::Named(NamedKey::F1) {
                        ui_pages.switch_page(PagesEnum::Help);
                        window.request_redraw();
                        println!("open help page");
                    } else if event.logical_key == Key::Named(NamedKey::F5) {
                        if modifiers.state() == ModifiersState::SHIFT {
                            println!("open preferences page")
                        } else if modifiers.state().is_empty() {
                            println!("open info page");
                        }
                    } else if event.logical_key == Key::Named(NamedKey::F12) {
                        if modifiers.state().is_empty() {
                            ui_pages.switch_page(PagesEnum::SongDirectoryConfig);
                            window.request_redraw();
                        }
                    } else {
                        ui_pages.process_key_event(&modifiers, event);
                        // maybe do this more efficently by only requesting when actually something changes
                        window.request_redraw();
                    }
                }
            }
            // not sure if i need it just to make sure i always have all current modifiers to be used with keyboard events
            WindowEvent::ModifiersChanged(new_modifiers) => modifiers = *new_modifiers,
            _ => {}
        },
        Event::UserEvent(()) => (),
        // runs before updating the screen again, so the pages are on current state, currently panics
        // Event::NewEvents(_) => pages.update(),
        _ => {}
    });
}
