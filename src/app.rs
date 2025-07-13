use std::{
    collections::VecDeque,
    fmt::Debug,
    num::NonZero,
    sync::{Arc, LazyLock, OnceLock},
    thread::JoinHandle,
    time::Duration,
};

use smol::{channel::Sender, lock::Mutex};
use torque_tracker_engine::{
    audio_processing::playback::PlaybackStatus,
    manager::{AudioManager, OutputConfig, PlaybackSettings, SendResult, ToWorkerMsg},
    project::song::{Song, SongOperation},
};
use triple_buffer::triple_buffer;
use winit::{
    application::ApplicationHandler,
    event::{Modifiers, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

use cpal::{
    BufferSize, SupportedBufferSize,
    traits::{DeviceTrait, HostTrait},
};

use crate::{
    palettes::Palette,
    ui::pages::{order_list::OrderListPageEvent, pattern::PatternPageEvent},
};

use super::{
    draw_buffer::DrawBuffer,
    render::RenderBackend,
    ui::{
        dialog::{
            Dialog, DialogManager, DialogResponse, confirm::ConfirmDialog, page_menu::PageMenu,
        },
        header::{Header, HeaderEvent},
        pages::{AllPages, PageEvent, PageResponse, PagesEnum},
    },
};

pub static EXECUTOR: smol::Executor = smol::Executor::new();
pub static AUDIO: LazyLock<Mutex<AudioManager>> =
    LazyLock::new(|| Mutex::new(AudioManager::new(Song::default())));
pub static SONG_OP_SEND: OnceLock<smol::channel::Sender<SongOperation>> = OnceLock::new();

/// shorter function name
pub fn send_song_op(op: SongOperation) {
    SONG_OP_SEND.get().unwrap().send_blocking(op).unwrap();
}

pub enum GlobalEvent {
    OpenDialog(Box<dyn FnOnce() -> Box<dyn Dialog> + Send>),
    Page(PageEvent),
    Header(HeaderEvent),
    /// also closes all dialogs
    GoToPage(PagesEnum),
    CloseRequested,
    CloseApp,
    ConstRedraw,
}

impl Debug for GlobalEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("GlobalEvent");
        match self {
            GlobalEvent::OpenDialog(_) => debug.field("OpenDialog", &"closure"),
            GlobalEvent::Page(page_event) => debug.field("Page", page_event),
            GlobalEvent::Header(header_event) => debug.field("Header", header_event),
            GlobalEvent::GoToPage(pages_enum) => debug.field("GoToPage", pages_enum),
            GlobalEvent::CloseRequested => debug.field("CloseRequested", &""),
            GlobalEvent::CloseApp => debug.field("CloseApp", &""),
            GlobalEvent::ConstRedraw => debug.field("ConstRedraw", &""),
        };
        debug.finish()
    }
}

struct WorkerThreads {
    handles: [JoinHandle<()>; 2],
    close_msg: [Sender<()>; 2],
}

impl WorkerThreads {
    fn new() -> Self {
        let (send1, recv1) = smol::channel::unbounded();
        let thread1 = std::thread::Builder::new()
            .name("Background Worker 1".into())
            .spawn(Self::worker_task(recv1))
            .unwrap();
        let (send2, recv2) = smol::channel::unbounded();
        let thread2 = std::thread::Builder::new()
            .name("Background Worker 2".into())
            .spawn(Self::worker_task(recv2))
            .unwrap();

        Self {
            handles: [thread1, thread2],
            close_msg: [send1, send2],
        }
    }

    fn worker_task(recv: smol::channel::Receiver<()>) -> impl FnOnce() + Send + 'static {
        move || {
            smol::block_on(EXECUTOR.run(async { recv.recv().await.unwrap() }));
        }
    }

    /// prepares the closing of the threads by signalling them to stop
    fn send_close(&mut self) {
        _ = self.close_msg[0].send_blocking(());
        _ = self.close_msg[1].send_blocking(());
    }

    fn close_all(mut self) {
        self.send_close();
        let [handle1, handle2] = self.handles;
        handle1.join().unwrap();
        handle2.join().unwrap();
    }
}

pub struct App {
    window_gpu: Option<(Arc<Window>, RenderBackend)>,
    draw_buffer: DrawBuffer,
    modifiers: Modifiers,
    ui_pages: AllPages,
    event_queue: VecDeque<GlobalEvent>,
    dialog_manager: DialogManager,
    header: Header,
    event_loop_proxy: EventLoopProxy<GlobalEvent>,
    worker_threads: Option<WorkerThreads>,
    audio_stream: Option<(
        cpal::Stream,
        triple_buffer::Output<Option<cpal::OutputStreamTimestamp>>,
        smol::Task<()>,
    )>,
}

impl ApplicationHandler<GlobalEvent> for App {
    fn new_events(&mut self, _: &ActiveEventLoop, start_cause: winit::event::StartCause) {
        if start_cause == winit::event::StartCause::Init {
            LazyLock::force(&AUDIO);
            self.worker_threads = Some(WorkerThreads::new());
            let (send, recv) = smol::channel::unbounded();
            SONG_OP_SEND.get_or_init(|| send);
            EXECUTOR
                .spawn(async move {
                    while let Ok(op) = recv.recv().await {
                        let mut manager = AUDIO.lock().await;

                        loop {
                            if let Some(mut song) = manager.try_edit_song() {
                                song.apply_operation(op).unwrap();
                                // don't need to relock if there are more operations in queue
                                while let Ok(op) = recv.try_recv() {
                                    song.apply_operation(op).unwrap();
                                }
                                break;
                            }
                            let buffer_time = manager.buffer_time().expect("locking failed once, so audio must be active, so there must be a buffer_time");
                            smol::Timer::after(buffer_time).await;
                        }
                    }
                })
                .detach();
            // spawn a task to collect sample garbage every 10 seconds
            EXECUTOR
                .spawn(async {
                    loop {
                        let mut lock = AUDIO.lock().await;
                        lock.collect_garbage();
                        drop(lock);
                        smol::Timer::after(Duration::from_secs(10)).await;
                    }
                })
                .detach();
            self.start_audio_stream();
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
            event_queue,
            dialog_manager,
            header,
            event_loop_proxy: _,
            worker_threads: _,
            audio_stream: _,
        } = self;

        // panic is fine because when i get a window_event a window exists
        let (window, render_backend) = window_gpu.as_mut().unwrap();
        // don't want the window to be mut
        let window = window.as_ref();

        match event {
            WindowEvent::CloseRequested => Self::close_requested(event_queue),
            WindowEvent::Resized(physical_size) => {
                render_backend.resize(physical_size);
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _,
            } => {
                // window_state.resize(**new_inner_size);
                // due to a version bump in winit i dont know anymore how to handle this event so i just ignore it for know and see if it makes problems in the future
                // i have yet only received this event on linux wayland, not macos
                println!("Window Scale Factor Changed");
            }
            WindowEvent::RedrawRequested => {
                // draw the new frame buffer
                // TODO: split redraw header and redraw page. As soon as header gets a spectrometer this becomes important
                header.draw(draw_buffer);
                ui_pages.draw(draw_buffer);
                dialog_manager.draw(draw_buffer);
                // notify the windowing system that drawing is done and the new buffer is about to be pushed
                window.pre_present_notify();
                // push the framebuffer into GPU/softbuffer and render it onto the screen
                render_backend.render(&draw_buffer.framebuffer, event_loop);
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic,
            } => {
                if is_synthetic {
                    return;
                }

                let message = if event.logical_key == Key::Named(NamedKey::F5) {
                    // play song from start
                    Some(ToWorkerMsg::Playback(PlaybackSettings::Order {
                        idx: 0,
                        should_loop: true,
                    }))
                } else if event.logical_key == Key::Named(NamedKey::F6) {
                    Some(ToWorkerMsg::Playback(if modifiers.state().shift_key() {
                        // play the current order
                        self.header.play_current_order()
                    } else {
                        // play the current pattern
                        self.header.play_current_pattern()
                    }))
                // TODO: add F7 handling
                } else if event.logical_key == Key::Named(NamedKey::F8) {
                    Some(ToWorkerMsg::StopPlayback)
                } else {
                    None
                };
                if let Some(msg) = message {
                    let result = AUDIO.lock_blocking().try_msg_worker(msg);

                    match result {
                        SendResult::Success => (),
                        SendResult::BufferFull => {
                            panic!("to worker buffer full, probably have to retry somehow")
                        }
                        SendResult::AudioInactive => panic!("audio should always be active"),
                    }
                } else {
                    // key_event didn't start or stop the song, so process normally
                    if let Some(dialog) = dialog_manager.active_dialog_mut() {
                        match dialog.process_input(&event, modifiers, event_queue) {
                            DialogResponse::Close => {
                                dialog_manager.close_dialog();
                                // if i close a pop_up i need to redraw the const part of the page as the pop-up overlapped it probably
                                ui_pages.request_draw_const();
                                window.request_redraw();
                            }
                            DialogResponse::RequestRedraw => window.request_redraw(),
                            DialogResponse::None => (),
                        }
                    } else {
                        if event.state.is_pressed()
                            && event.logical_key == Key::Named(NamedKey::Escape)
                        {
                            event_queue.push_back(GlobalEvent::OpenDialog(Box::new(|| {
                                Box::new(PageMenu::main())
                            })));
                        }

                        match ui_pages.process_key_event(&self.modifiers, &event, event_queue) {
                            PageResponse::RequestRedraw => window.request_redraw(),
                            PageResponse::None => (),
                        }
                    }
                }
            }
            // not sure if i need it just to make sure i always have all current modifiers to be used with keyboard events
            WindowEvent::ModifiersChanged(new_modifiers) => *modifiers = new_modifiers,

            _ => (),
        }

        while let Some(event) = self.event_queue.pop_front() {
            self.user_event(event_loop, event);
        }
    }

    /// i may need to add the ability for events to add events to the event queue, but that should be possible
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: GlobalEvent) {
        match event {
            GlobalEvent::OpenDialog(dialog) => {
                self.dialog_manager.open_dialog(dialog());
                _ = self.try_request_redraw();
            }
            GlobalEvent::Page(c) => {
                match self.ui_pages.process_page_event(c, &mut self.event_queue) {
                    PageResponse::RequestRedraw => _ = self.try_request_redraw(),
                    PageResponse::None => (),
                }
            }
            GlobalEvent::Header(header_event) => {
                self.header.process_event(header_event);
                _ = self.try_request_redraw();
            }
            GlobalEvent::GoToPage(pages_enum) => {
                self.dialog_manager.close_all();
                self.ui_pages.switch_page(pages_enum);
                _ = self.try_request_redraw();
            }
            GlobalEvent::CloseApp => event_loop.exit(),
            GlobalEvent::CloseRequested => Self::close_requested(&mut self.event_queue),
            GlobalEvent::ConstRedraw => {
                self.ui_pages.request_draw_const();
                _ = self.try_request_redraw();
            }
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        if let Some(workers) = self.worker_threads.take() {
            // wait for all the threads to close
            workers.close_all();
        }
        if self.audio_stream.is_some() {
            self.close_audio_stream();
        }
    }
}

impl App {
    pub fn new(proxy: EventLoopProxy<GlobalEvent>) -> Self {
        Self {
            window_gpu: None,
            draw_buffer: DrawBuffer::new(),
            modifiers: Modifiers::default(),
            ui_pages: AllPages::new(proxy.clone()),
            dialog_manager: DialogManager::new(),
            header: Header::default(),
            event_loop_proxy: proxy,
            worker_threads: None,
            audio_stream: None,
            event_queue: VecDeque::with_capacity(3),
        }
    }

    fn close_requested(events: &mut VecDeque<GlobalEvent>) {
        events.push_back(GlobalEvent::OpenDialog(Box::new(|| {
            Box::new(ConfirmDialog::new(
                "Close Torque Tracker?",
                || Some(GlobalEvent::CloseApp),
                || None,
            ))
        })));
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
            attributes.title = String::from("Torque Tracker");

            let window = Arc::new(event_loop.create_window(attributes).unwrap());
            let render_backend = RenderBackend::new(window.clone(), Palette::CAMOUFLAGE);
            (window, render_backend)
        });
    }

    // TODO: make this configurable
    fn start_audio_stream(&mut self) {
        assert!(self.audio_stream.is_none());
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let default_config = device.default_output_config().unwrap();
        let (config, buffer_size) = {
            let mut config = default_config.config();
            let buffer_size = {
                let default = default_config.buffer_size();
                match default {
                    SupportedBufferSize::Unknown => 1024,
                    SupportedBufferSize::Range { min, max } => u32::min(u32::max(1024, *min), *max),
                }
            };
            config.buffer_size = BufferSize::Fixed(buffer_size);
            (config, buffer_size)
        };
        let mut guard = AUDIO.lock_blocking();
        let mut worker = guard.get_callback::<f32>(OutputConfig {
            buffer_size,
            channel_count: NonZero::new(config.channels).unwrap(),
            sample_rate: NonZero::new(config.sample_rate.0).unwrap(),
        });
        let buffer_time = guard.buffer_time().unwrap();
        // keep the guard as short as possible to not block the async threads
        drop(guard);
        let (mut send, recv) = triple_buffer(&None);
        let stream = device
            .build_output_stream(
                &config,
                move |data, info| {
                    worker(data);
                    send.write(Some(info.timestamp()));
                },
                |err| eprintln!("audio stream err: {err:?}"),
                None,
            )
            .unwrap();
        // spawn a task to process the audio playback status updates
        let proxy = self.event_loop_proxy.clone();
        let task = EXECUTOR.spawn(async move {
            let mut buffer_time = buffer_time;
            let mut old_status: Option<PlaybackStatus> = None;
            loop {
                let mut lock = AUDIO.lock().await;
                let status = lock.playback_status().cloned();
                let time = lock.buffer_time();
                drop(lock);
                let status = status.expect("background task running while no stream active");
                // only react on status changes. could at some point be made more granular
                if status != old_status {
                    old_status = status;
                    // println!("playback status: {status:?}");
                    let pos = status.map(|s| s.position);
                    proxy
                        .send_event(GlobalEvent::Header(HeaderEvent::SetPlayback(pos)))
                        .unwrap();
                    let pos = status.map(|s| (s.position.pattern, s.position.row));
                    proxy
                        .send_event(GlobalEvent::Page(PageEvent::Pattern(
                            PatternPageEvent::PlaybackPosition(pos),
                        )))
                        .unwrap();
                    // does a map flatten. idk why it's called and_then
                    let pos = status.and_then(|s| s.position.order);
                    proxy
                        .send_event(GlobalEvent::Page(PageEvent::OrderList(
                            OrderListPageEvent::SetPlayback(pos),
                        )))
                        .unwrap();
                }

                if let Some(time) = time {
                    assert!(time == buffer_time);
                    buffer_time = time;
                }
                smol::Timer::after(buffer_time).await;
            }
        });
        self.audio_stream = Some((stream, recv, task));
    }

    fn close_audio_stream(&mut self) {
        _ = self.audio_stream.take().unwrap();
        AUDIO.lock_blocking().stream_closed();
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if self.audio_stream.is_some() {
            self.close_audio_stream();
        }
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
