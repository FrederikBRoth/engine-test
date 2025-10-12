use sparmos_engine::cgmath::Vector2;

pub enum InputType {
    Insert,
    Delete,
}

pub struct Life {
    pub game_area: Vec<u8>,
    width: i32,
    height: i32,
    elapsed_time: f32,
    fps: f32,
    pub enabled: bool,
    pub input: Option<InputType>,
}

impl Life {
    pub fn new(initial_state: Vec<u8>, width: i32, height: i32, fps: f32) -> Self {
        Self {
            game_area: initial_state,
            width,
            height,
            elapsed_time: 0.0,
            fps: 1.0 / fps,
            enabled: true,
            input: None,
        }
    }

    fn get_toroidal_element(&self, x: i32, y: i32) -> u8 {
        let width = self.width as i32;
        let height = self.height as i32;

        let wrapped_x = (x % width + width) % width;
        let wrapped_y = (y % height + height) % height;

        // Flatten back
        let index = (wrapped_y * width + wrapped_x) as usize;

        //This wont panic due to how the index is calculated to wrap around the edges
        self.game_area[index]
    }

    pub fn neighboor_sum(&self, x: i32, y: i32) -> u8 {
        let current = self.get_toroidal_element(x, y);
        let top = self.get_toroidal_element(x, y - 1);
        let bottom = self.get_toroidal_element(x, y + 1);
        let right = self.get_toroidal_element(x + 1, y);
        let left = self.get_toroidal_element(x - 1, y);
        let top_right = self.get_toroidal_element(x + 1, y - 1);
        let top_left = self.get_toroidal_element(x - 1, y - 1);
        let bottom_right = self.get_toroidal_element(x + 1, y + 1);
        let bottom_left = self.get_toroidal_element(x - 1, y + 1);

        let sum = current
            + top
            + bottom
            + right
            + left
            + top_right
            + top_left
            + bottom_left
            + bottom_right;
        sum
    }

    pub fn calculate_iteration(&mut self, dt: std::time::Duration) {
        if !self.enabled {
            return;
        }
        let dts = dt.as_secs_f32();

        self.elapsed_time += dts;
        if self.elapsed_time >= self.fps {
            let new_iteration: Vec<u8> = self
                .game_area
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let x = i % self.width as usize;
                    let y = i / self.width as usize;
                    let sum = self.neighboor_sum(x as i32, y as i32);
                    if sum == 3 {
                        1
                    } else if sum == 4 {
                        *field
                    } else {
                        0
                    }
                })
                .collect();

            self.game_area = new_iteration;
            self.elapsed_time -= self.fps;
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn toggle_input(&mut self, input_type: Option<InputType>) {
        self.input = input_type;
    }

    pub fn reset_time(&mut self) {
        self.elapsed_time = 0.0;
    }
}
