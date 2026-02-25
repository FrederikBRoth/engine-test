pub mod app;
pub mod gameloop;
pub mod gameoflife;
pub mod gui;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use sparmos_engine::{core::state::GameLoop, prelude::run_game, wgpu, winit};

// use app; // Removed because there is no external crate or module named 'app'

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    use crate::{
        app::{MyGame, WasmEvent},
        gameloop::MyLoop,
    };

    console_error_panic_hook::set_once();
    run_game::<WasmEvent, _, MyLoop>(
        MyGame { score: 0 },
        MyLoop {
            score: 0,
            instance_controllers: vec![],
            camera_controller: None,
            ..Default::default()
        },
    )
    .unwrap_throw();
    Ok(())
}
