use engine_test::{
    app::{MyGame, WasmEvent},
    gameloop::MobiusVisualizer,
};
use sparmos_engine::prelude::run_game;

fn main() {
    run_game::<WasmEvent, _, MobiusVisualizer>(
        MyGame { score: 0 },
        MobiusVisualizer {
            score: 0,
            ..Default::default()
        },
    )
    .unwrap();
}
