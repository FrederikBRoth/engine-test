#[cfg(target_arch = "wasm32")]
use sparmos_engine::{core::event_loop::UserEvent, winit};
use sparmos_engine::{
    core::{event_loop::AppLifecycle, state::State},
    log,
    winit::event::DeviceEvent,
};

use crate::gameloop::MyLoop;

pub enum WasmEvent {
    ScrollPosition { x: f64, y: f64 },
    KeyboardButton { keypress: String },
}
pub struct MyGame {
    pub score: u32,
}

impl AppLifecycle<WasmEvent, MyLoop> for MyGame {
    fn on_user_event(&mut self, state: &mut State<MyLoop>, event: WasmEvent) {
        match event {
            WasmEvent::ScrollPosition { x, y } => log::warn!("x: {}, y: {}", x, y),
            WasmEvent::KeyboardButton { keypress } => log::warn!("keypress: {}", keypress),
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn on_resumed(
        &mut self,
        proxy: &winit::event_loop::EventLoopProxy<UserEvent<WasmEvent, MyLoop>>,
    ) {
        use sparmos_engine::wgpu;
        use wasm_bindgen::{JsCast, prelude::Closure};

        let window = wgpu::web_sys::window().unwrap();
        let window_clone = window.clone();

        let p = proxy.clone();
        let closure2 = Closure::<dyn FnMut(_)>::new(move |_event: wgpu::web_sys::Event| {
            let x = window_clone.scroll_x().unwrap_or(0.0);
            let y = window_clone.scroll_y().unwrap_or(0.0);

            // Send a custom event with the scroll data to your app
            let _ = p.send_event(UserEvent::Custom(WasmEvent::ScrollPosition { x, y }));
        });

        window
            .add_event_listener_with_callback("scroll", closure2.as_ref().unchecked_ref())
            .unwrap();

        closure2.forget();

        let p = proxy.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |event: wgpu::web_sys::Event| {
            use web_sys::KeyboardEvent;

            if let Some(kev) = event.dyn_ref::<KeyboardEvent>() {
                let _ = p.send_event(UserEvent::Custom(WasmEvent::KeyboardButton {
                    keypress: kev.key(),
                }));
            }
            // else: it's some other kind of Event, ignore it
        });

        window
            .add_event_listener_with_callback("keypress", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn on_resumed(&mut self) {
        println!("Dank")
    }

    fn on_device_event(&mut self, event: DeviceEvent, proxy: &mut State<MyLoop>) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                proxy.game_loop.as_mut().unwrap().cursor_delta = delta;
            }
            _ => (),
        }
    }
}
