use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes};

/// A desktop window with a software framebuffer.
///
/// Write ARGB32 pixels into [`Self::buffer_mut()`] and call
/// [`Self::present()`] to display them.
pub struct DesktopSurface {
    window: std::rc::Rc<Window>,
    surf: Surface<std::rc::Rc<Window>, std::rc::Rc<Window>>,
    width: u32,
    height: u32,
}

impl DesktopSurface {
    /// Run a closure inside a winit event loop.
    ///
    /// The closure is called for each event. Return `true` to
    /// continue, `false` to exit.
    pub fn run(
        title: &str,
        width: u32,
        height: u32,
        frame_fn: impl FnMut(&mut DesktopSurface, Event) -> bool,
    ) {
        struct Handler<F: FnMut(&mut DesktopSurface, Event) -> bool> {
            surface: Option<DesktopSurface>,
            f: F,
            title: String,
            w: u32,
            h: u32,
        }

        impl<F: FnMut(&mut DesktopSurface, Event) -> bool> ApplicationHandler for Handler<F> {
            fn resumed(&mut self, el: &ActiveEventLoop) {
                let window = std::rc::Rc::new(
                    el.create_window(
                        WindowAttributes::default()
                            .with_title(&self.title)
                            .with_inner_size(LogicalSize::new(self.w, self.h)),
                    )
                    .expect("window"),
                );
                let ctx = Context::new(window.clone()).expect("context");
                let surf = Surface::new(&ctx, window.clone()).expect("surface");
                self.surface = Some(DesktopSurface {
                    window,
                    surf,
                    width: self.w,
                    height: self.h,
                });
                el.set_control_flow(ControlFlow::Poll);
            }

            fn window_event(
                &mut self,
                el: &ActiveEventLoop,
                _: winit::window::WindowId,
                event: WindowEvent,
            ) {
                let s = self.surface.as_mut().unwrap();
                let ev = match event {
                    WindowEvent::CloseRequested => Event::Close,
                    WindowEvent::Resized(size) => {
                        s.width = size.width.max(1);
                        s.height = size.height.max(1);
                        let _ = s
                            .surf
                            .resize(
                                std::num::NonZeroU32::new(s.width).unwrap(),
                                std::num::NonZeroU32::new(s.height).unwrap(),
                            );
                        Event::Resize(s.width, s.height)
                    }
                    WindowEvent::KeyboardInput { event, .. } => Event::Key(event),
                    WindowEvent::RedrawRequested => Event::Draw,
                    _ => return,
                };
                if !(self.f)(s, ev) {
                    el.exit();
                }
            }

            fn about_to_wait(&mut self, _: &ActiveEventLoop) {
                if let Some(s) = &self.surface {
                    s.window.request_redraw();
                }
            }
        }

        let mut h = Handler {
            surface: None,
            f: frame_fn,
            title: title.to_string(),
            w: width,
            h: height,
        };
        let el = EventLoop::new().unwrap();
        el.set_control_flow(ControlFlow::Poll);
        el.run_app(&mut h).unwrap();
    }

    /// Get the pixel buffer as ARGB32.
    pub fn buffer_mut(&mut self) -> &mut [u32] {
        match self.surf.buffer_mut() {
            Ok(mut buf) => {
                let len = buf.len() / 4;
                let ptr = buf.as_mut_ptr() as *mut u32;
                unsafe { std::slice::from_raw_parts_mut(ptr, len) }
            }
            Err(_) => &mut [],
        }
    }

    /// Present the buffer to the window.
    pub fn present(&mut self) {
        if let Ok(buf) = self.surf.buffer_mut() {
            let _ = buf.present();
        }
    }

    /// Current window dimensions.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Access the winit window.
    pub fn window(&self) -> &Window {
        &self.window
    }
}

/// Events delivered to the frame callback.
pub enum Event {
    Draw,
    Resize(u32, u32),
    Key(winit::event::KeyEvent),
    Close,
}
