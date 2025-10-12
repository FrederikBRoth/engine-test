use engine_test::{
    app::{MyGame, WasmEvent},
    gameloop::MyLoop,
};
use sparmos_engine::prelude::run_game;

fn main() {
    run_game::<WasmEvent, _, MyLoop>(
        MyGame { score: 0 },
        MyLoop {
            score: 0,
            instance_controllers: vec![],
            camera_controller: None,
            ..Default::default()
        },
    )
    .unwrap();
}
