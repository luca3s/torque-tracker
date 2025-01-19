use std::{
    mem::ManuallyDrop,
    sync::{Arc, LazyLock},
    thread::JoinHandle,
};

use smol::{channel::Sender, lock::Mutex};
use tracker_engine::playback::audio_manager::AudioManager;
use winit::{
    application::ApplicationHandler,
    event::{Modifiers, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

use super::{
    draw_buffer::DrawBuffer,
    gpu::GPUState,
    ui::{
        dialog::{page_menu::PageMenu, Dialog, DialogManager, DialogResponse},
        header::Header,
        pages::{AllPages, PageEvent, PageResponse},
    },
};

static EXECUTOR: smol::Executor = smol::Executor::new();
static AUDIO: LazyLock<Mutex<AudioManager>> = LazyLock::new(|| Mutex::new(AudioManager::new()));

pub enum GlobalEvent {
    OpenDialog(Box<dyn Dialog>),
    PageEvent(PageEvent),
}

struct WorkerThreads {
    standard: JoinHandle<()>,
    audio_extra: Option<(JoinHandle<()>, Sender<()>)>,
    close_msg: Sender<()>,
}

impl WorkerThreads {
    fn new() -> Self {
        let (send, recv) = smol::channel::unbounded();
        let thread = std::thread::Builder::new()
            .name("Background Worker".into())
            .spawn(Self::worker_task(recv))
            .unwrap();

        Self {
            standard: thread,
            audio_extra: None,
            close_msg: send,
        }
    }

    fn worker_task(recv: smol::channel::Receiver<()>) -> impl FnOnce() + Send + 'static {
        move || {
            smol::block_on(EXECUTOR.run(async { recv.recv().await.unwrap() }));
        }
    }

    fn start_audio(&mut self) {
        self.audio_extra.get_or_insert_with(|| {
            let (send, recv) = smol::channel::unbounded();
            let thread = std::thread::Builder::new()
                .name("2nd Background Worker".into())
                .spawn(Self::worker_task(recv))
                .unwrap();

            (thread, send)
        });
    }

    /// signals the audio thread to stop and waits until it does
    fn close_audio(&mut self) {
        if let Some((thread, send)) = self.audio_extra.take() {
            _ = send.send_blocking(());
            send.close();
            thread.join().unwrap();
        }
    }

    /// prepares the closing of the threads by signalling them to stop
    fn send_close(&mut self) {
        _ = self.close_msg.send_blocking(());
        self.close_audio();
    }

    fn close_all(mut self) {
        self.send_close();
        self.close_audio();
        self.standard.join().unwrap();
    }
}

pub struct App {
    window_gpu: Option<(Arc<Window>, GPUState)>,
    draw_buffer: DrawBuffer,
    modifiers: Modifiers,
    ui_pages: AllPages,
    dialog_manager: DialogManager,
    header: Header,
    event_loop_proxy: EventLoopProxy<GlobalEvent>,
    worker_threads: Option<WorkerThreads>,
}

impl ApplicationHandler<GlobalEvent> for App {
    fn new_events(&mut self, _: &ActiveEventLoop, start_cause: winit::event::StartCause) {
        if start_cause == winit::event::StartCause::Init {
            // LazyLock::force(&AUDIO);
            self.worker_threads = Some(WorkerThreads::new())
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.build_window(event_loop);
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        // my window and GPU state have been invalidated
        self.window_gpu = None;
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: WindowEvent,
    ) {
        // destructure so i don't have to always type self.
        let Self {
            window_gpu,
            draw_buffer,
            modifiers,
            ui_pages,
            dialog_manager,
            header: _,
            event_loop_proxy: _,
            worker_threads: _,
        } = self;

        // panic is fine because when i get a window_event a window exists
        let (window, gpu_state) = window_gpu.as_mut().unwrap();
        // don't want the window to be mut
        let window = window.as_ref();

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(physical_size) => {
                gpu_state.resize(physical_size);
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
                ui_pages.draw(draw_buffer);
                dialog_manager.draw(draw_buffer);
                // notify the windowing system that drawing is done and the new buffer is about to be pushed
                window.pre_present_notify();
                // push the framebuffer into GPU and render it onto the screen
                match gpu_state.render(&draw_buffer.framebuffer) {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => gpu_state.reinit_surface(),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => eprint!("{:?}", e),
                }
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => {
                if is_synthetic {
                    return;
                }

                if let Some(dialog) = dialog_manager.active_dialog_mut() {
                    match dialog.process_input(&event, modifiers) {
                        DialogResponse::Close => {
                            dialog_manager.close_dialog();
                            // if i close a pop_up i need to redraw the const part of the page as the pop-up overlapped it probably
                            ui_pages.request_draw_const();
                            window.request_redraw();
                        }
                        DialogResponse::RequestRedraw => window.request_redraw(),
                        DialogResponse::None => (),
                        DialogResponse::SwitchToPage(page) => {
                            dialog_manager.close_all();
                            ui_pages.switch_page(page);
                            window.request_redraw();
                        }
                        DialogResponse::GlobalEvent(e, should_close) => {
                            if should_close {
                                dialog_manager.close_dialog();
                                ui_pages.request_draw_const();
                                window.request_redraw();
                            }
                            self.user_event(event_loop, e);
                        }
                    }
                } else {
                    if event.state.is_pressed() && event.logical_key == Key::Named(NamedKey::Escape)
                    {
                        let main_menu = PageMenu::main();
                        self.user_event(event_loop, GlobalEvent::OpenDialog(Box::new(main_menu)));
                    }

                    match self.ui_pages.process_key_event(&self.modifiers, &event) {
                        PageResponse::RequestRedraw => _ = self.try_request_redraw(),
                        PageResponse::None => (),
                        // if the page wants to send a custom_event i don't really have to send it i can just handle it myself
                        PageResponse::GlobalEvent(event) => self.user_event(event_loop, event),
                    }
                }
            }
            // not sure if i need it just to make sure i always have all current modifiers to be used with keyboard events
            WindowEvent::ModifiersChanged(new_modifiers) => *modifiers = new_modifiers,

            _ => (),
        }
    }

    /// loops while the response the the event is a new event and processes that
    fn user_event(&mut self, _: &ActiveEventLoop, mut event: GlobalEvent) {
        loop {
            match event {
                GlobalEvent::OpenDialog(dialog) => {
                    self.dialog_manager.open_dialog(dialog);
                    _ = self.try_request_redraw();
                    break;
                }
                GlobalEvent::PageEvent(c) => {
                    match self.ui_pages.process_page_event(c) {
                        PageResponse::RequestRedraw => {
                            // i may get a user_event without an existing window
                            _ = self.try_request_redraw();
                            break;
                        }
                        PageResponse::None => break,
                        PageResponse::GlobalEvent(new_event) => event = new_event,
                    }
                }
            }
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        if let Some(workers) = self.worker_threads.take() {
            // wait for all the threads to close
            workers.close_all();
        }
    }
}

impl App {
    pub fn new(proxy: EventLoopProxy<GlobalEvent>) -> Self {
        Self {
            window_gpu: None,
            draw_buffer: DrawBuffer::new(),
            modifiers: Modifiers::default(),
            ui_pages: AllPages::new(),
            dialog_manager: DialogManager::new(),
            header: Header {},
            event_loop_proxy: proxy,
            worker_threads: None,
        }
    }

    /// tries to request a redraw. if there currently is no window this fails
    fn try_request_redraw(&self) -> Result<(), ()> {
        if let Some((window, _)) = &self.window_gpu {
            window.request_redraw();
            Ok(())
        } else {
            Err(())
        }
    }

    fn build_window(&mut self, event_loop: &ActiveEventLoop) {
        self.window_gpu.get_or_insert_with(|| {
            let mut attributes = WindowAttributes::default();
            attributes.active = true;
            attributes.resizable = true;
            attributes.resize_increments = None;
            attributes.title = String::from("RusTracker");

            let window = Arc::new(event_loop.create_window(attributes).unwrap());
            let gpu_state = smol::block_on(GPUState::new(window.clone()));
            (window, gpu_state)
        });
    }
}

pub fn run() {
    let event_loop = winit::event_loop::EventLoop::<GlobalEvent>::with_user_event()
        .build()
        .unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);
    // i don't need any raw device events. Keyboard and Mouse coming as window events are enough
    event_loop.listen_device_events(winit::event_loop::DeviceEvents::Never);
    let event_loop_proxy = event_loop.create_proxy();
    let mut app = App::new(event_loop_proxy);
    app.header.draw_constant(&mut app.draw_buffer);

    event_loop.run_app(&mut app).unwrap();
}
