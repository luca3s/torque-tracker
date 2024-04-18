use winit::{
    event::{Event, Modifiers, WindowEvent},
    event_loop::EventLoopBuilder,
    keyboard::{Key, ModifiersState, NamedKey},
    window::Window,
};

use crate::main;

use super::{
    draw_buffer::DrawBuffer,
    gpu::GPUState,
    ui::{
        dialog::{
            dialog::{Dialog, DialogResponse, DialogState},
            page_menu::PageMenu,
        },
        header::Header,
        pages::page::{AllPages, Page, PageResponse},
    },
};

pub enum CustomWinitEvent {
    OpenDialog(Box<dyn Dialog>),
}

pub fn run() {
    let event_loop = winit::event_loop::EventLoopBuilder::<CustomWinitEvent>::with_user_event()
        .build()
        .unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
    let event_loop_proxy = event_loop.create_proxy();

    let window = Window::new(&event_loop).unwrap();

    let mut gpu_state = pollster::block_on(GPUState::new(&window));

    let mut draw_buffer = DrawBuffer::new();
    let mut modifiers = Modifiers::default();
    let mut ui_pages = AllPages::new(event_loop_proxy.clone());
    let mut dialog_state = DialogState::new();

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
                ui_pages.update();

                // draw the new frame buffer
                ui_pages.draw(&mut draw_buffer);

                dialog_state.draw(&mut draw_buffer);
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
                if *is_synthetic {
                    return;
                }

                if let Some(dialog) = dialog_state.active_dialog_mut() {
                    match dialog.process_input(event, &modifiers) {
                        DialogResponse::Close => {
                            dialog_state.close_dialog();
                            // if i close a pop_up i need to redraw the const part of the page as the pop-up overlapped it probably
                            ui_pages.request_draw_const();
                            window.request_redraw();
                        }
                        DialogResponse::RequestRedraw => window.request_redraw(),
                        DialogResponse::None => (),
                        DialogResponse::SwitchToPage(page) => {
                            dialog_state.close_all();
                            ui_pages.switch_page(page);
                            window.request_redraw();
                        },
                    }
                } else {
                    if event.state.is_pressed() && event.logical_key == Key::Named(NamedKey::Escape)
                    {
                        let main_menu = PageMenu::main(event_loop_proxy.clone());
                        let _ = event_loop_proxy
                            .send_event(CustomWinitEvent::OpenDialog(Box::new(main_menu)));
                    }

                    match ui_pages.process_key_event(&modifiers, event) {
                        PageResponse::RequestRedraw => window.request_redraw(),
                        PageResponse::None => (),
                    }
                }
            }
            // not sure if i need it just to make sure i always have all current modifiers to be used with keyboard events
            WindowEvent::ModifiersChanged(new_modifiers) => modifiers = *new_modifiers,
            _ => {}
        },
        Event::UserEvent(event) => match event {
            CustomWinitEvent::OpenDialog(pop_up) => {
                dialog_state.open_dialog(pop_up);
                window.request_redraw();
            }
        },
        // runs before updating the screen again, so the pages are on current state, currently panics
        // Event::NewEvents(_) => pages.update(),
        _ => {}
    });
}
