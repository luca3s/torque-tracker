use winit::{
    event::{Event, Modifiers, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

use super::{
    draw_buffer::DrawBuffer,
    gpu::WindowState,
    ui::{
        header::Header,
        pages::page::{AllPages, Page},
    },
};

pub fn run(event_loop: EventLoop<()>, window: Window) {
    let mut window_state = pollster::block_on(WindowState::new(&window));
    let mut draw_buffer = DrawBuffer::new();
    let mut modifiers = Modifiers::default();
    let mut pages = AllPages::new();

    let ui_header = Header {};
    ui_header.draw_constant(&mut draw_buffer);
    // pages.draw_constant(&mut draw_buffer);
    let window = &window;
    let _ = event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            window_id: _, // can ignore because i only use one window
            ref event,
        } => match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::Resized(pyhsical_size) => {
                window_state.resize(*pyhsical_size);
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
                pages.draw(&mut draw_buffer);
                // draw_buffer.draw_rect(2, CharRect::new(15, 15, 1, 1));
                // push the framebuffer into GPU and render it onto the screen
                match window_state.render(draw_buffer.framebuffer.flatten()) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => window_state.resize(window_state.size),
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
                    pages.process_key_event(&modifiers, event);
                    window.request_redraw();
                }
            }
            // not sure if i need it just to make sure i always have all current modifiers to be used with keyboard events
            WindowEvent::ModifiersChanged(new_modifiers) => modifiers = *new_modifiers,
            _ => {}
        },
        Event::UserEvent(()) => (),
        _ => {}
    });
}
