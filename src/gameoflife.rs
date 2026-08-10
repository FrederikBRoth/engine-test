use std::{collections::HashSet, ptr::read};

use sparmos_engine::{
    cgmath::Vector2,
    core::geometry::{Primitive, PrimitiveVertex},
    wgpu,
};

pub enum InputType {
    Insert,
    Delete,
}

pub struct Life {
    pub game_area: Vec<u8>,
    pub width: i32,
    pub height: i32,
    elapsed_time: f32,
    fps: f32,
    pub enabled: bool,
    pub input: Option<InputType>,
    pub prev_drawed_elem: Option<usize>,
    pub lifeforms: Vec<Lifeform>,
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
            prev_drawed_elem: None,
            lifeforms: vec![],
        }
    }

    pub fn add_lifeform(&mut self, lifeform: Lifeform) {
        self.lifeforms.push(lifeform);
    }
    fn set_toroidal_element(&mut self, x: i32, y: i32, state: u8) {
        let width = self.width;
        let height = self.height;

        let wrapped_x = (x % width + width) % width;
        let wrapped_y = (y % height + height) % height;

        let index = (wrapped_y * width + wrapped_x) as usize;

        //This wont panic due to how the index is calculated to wrap around the edges
        self.game_area[index] = state;
    }

    fn get_toroidal_element(&self, x: i32, y: i32) -> u8 {
        let width = self.width;
        let height = self.height;

        let wrapped_x = (x % width + width) % width;
        let wrapped_y = (y % height + height) % height;

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

        current + top + bottom + right + left + top_right + top_left + bottom_left + bottom_right
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
                    let pos = get_grid_pos(i as i32, self.width);
                    let sum = self.neighboor_sum(pos.x, pos.y);
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

    pub fn convert_copy_01_to_0255(&mut self) -> Vec<u8> {
        self.game_area.iter().map(|&v| v * 255).collect()
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

    pub fn insert_premade_lifeform(&mut self, lifeform: Lifeform, middlepoint: usize) {
        let middle_pos = get_grid_pos(middlepoint as i32, self.width);

        let middle_x = lifeform.width as i32 / 2;
        let middle_y = lifeform.height as i32 / 2;
        for (i, data) in lifeform.data.iter().enumerate() {
            let pos = get_grid_pos(i as i32, lifeform.width as i32);

            self.set_toroidal_element(
                middle_pos.x + (pos.x - middle_x),
                middle_pos.y + (pos.y - middle_y),
                *data,
            );
        }
    }

    pub fn upload_to_texture(&mut self, queue: &wgpu::Queue, texture: &wgpu::Texture) {
        let readable = self.convert_copy_01_to_0255();

        let size = wgpu::Extent3d {
            width: self.width as u32,
            height: self.height as u32,
            depth_or_array_layers: 1,
        };

        queue.write_texture(
            texture.as_image_copy(),
            &readable,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width as u32),
                rows_per_image: Some(self.height as u32),
            },
            size,
        );
    }
}

#[derive(Clone)]
pub struct Lifeform {
    pub name: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MobiusSize {
    pub width: u32,
    pub height: u32,
}

impl Lifeform {
    pub fn new(name: String, width: u32, height: u32) -> Self {
        Self {
            name,
            data: vec![0; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn reset(&mut self, current_lifeform_count: usize) {
        self.name = format!("New Lifeform {}", current_lifeform_count);
        self.width = 0;
        self.height = 0;
        self.data = vec![];
    }
}

fn get_grid_pos(index: i32, width: i32) -> Vector2<i32> {
    Vector2 {
        x: index % width,
        y: index / width,
    }
}

pub fn get_neighbor_indices(index: usize, width: i32, height: i32, radius: i32) -> Vec<usize> {
    let x = (index as i32) % width;
    let y = (index as i32) / width;

    let wrap = |v: i32, max: i32| ((v % max) + max) % max;

    let mut neighbors = Vec::new();

    // Loop over all offsets within the orthogonal radius (square neighborhood)
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            // Skip the center cell itself
            if dx == 0 && dy == 0 {
                continue;
            }

            let nx = wrap(x + dx, width);
            let ny = wrap(y + dy, height);

            neighbors.push((ny * width + nx) as usize);
        }
    }

    neighbors
}
pub fn double_sided_mobius_strip(
    radius: f32,
    width: f32,
    num_grid_length: usize,
    num_grid_width: usize,
) -> Primitive {
    let (mut mobius_mesh, offset) =
        mobius_strip(radius, width, num_grid_length, num_grid_width, false, 0);
    let (mobius_mesh2, _) =
        mobius_strip(radius, width, num_grid_length, num_grid_width, true, offset);
    mobius_mesh.vertices.extend(mobius_mesh2.vertices);
    mobius_mesh.indices.extend(mobius_mesh2.indices);
    mobius_mesh
}

pub fn mobius_strip(
    radius: f32,
    width: f32,
    num_grid_length: usize,
    num_grid_width: usize,
    reversed: bool,
    index_offset: u32,
) -> (Primitive, u32) {
    let mut all_vertices = Vec::new();
    let mut all_indices = Vec::new();

    let mut colorb = true;

    // Position function for the Möbius strip
    let p = |u: f32, v: f32| -> [f32; 3] {
        let cu = u.cos();
        let su = u.sin();
        let cu2 = (u / 2.0).cos();
        let su2 = (u / 2.0).sin();
        [(radius + v * cu2) * cu, (radius + v * cu2) * su, v * su2]
    };

    let mut index_offset = index_offset;
    //This is because the index offset is incremented by the vertices size which is 4 in this scenario
    let mut quad_id = index_offset / 4;
    // Loop over grid cells
    for i in 0..num_grid_length {
        let i_next = (i + 1) % num_grid_length;
        let u0 = i as f32 / num_grid_length as f32 * 2.0 * std::f32::consts::PI;
        let u1 = i_next as f32 / num_grid_length as f32 * 2.0 * std::f32::consts::PI;
        let range: Box<dyn Iterator<Item = usize>> = if reversed {
            Box::new((0..(num_grid_width - 1)).rev())
        } else {
            Box::new(0..(num_grid_width - 1)) // or 0..=50 depending on inclusive/exclusive
        };

        for j in range {
            // Compute v coordinates
            let v0 = -width + j as f32 / (num_grid_width as f32 - 1.0) * 2.0 * width;
            let v1 = -width + (j + 1) as f32 / (num_grid_width as f32 - 1.0) * 2.0 * width;

            // Möbius flip at the seam
            let p00 = p(u0, v0);
            let p01 = p(u0, v1);
            let p10 = if i_next == 0 { p(u1, -v0) } else { p(u1, v0) };
            let p11 = if i_next == 0 { p(u1, -v1) } else { p(u1, v1) };

            // println!("ID: {:?}", quad_id,);
            let color = if colorb {
                [1.0, 0.0, 0.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            colorb = !colorb;

            // --- Shared vertices for this quad (4 unique vertices) ---
            let mut face_vertices = vec![
                PrimitiveVertex {
                    quad_id,
                    position: p00,
                    color,
                    normal: [0.0, 0.0, 0.0],
                },
                PrimitiveVertex {
                    quad_id,
                    position: p10,
                    color,
                    normal: [0.0, 0.0, 0.0],
                },
                PrimitiveVertex {
                    quad_id,
                    position: p11,
                    color,
                    normal: [0.0, 0.0, 0.0],
                },
                PrimitiveVertex {
                    quad_id,
                    position: p01,
                    color,
                    normal: [0.0, 0.0, 0.0],
                },
            ];

            // --- Indexed triangles (two per quad) ---
            let mut face_indices: Vec<u32> = if !reversed {
                vec![0, 1, 2, 0, 2, 3]
            } else {
                vec![0, 2, 1, 0, 3, 2]
            };

            // --- Compute averaged normals ---
            for tri in face_indices.chunks(3) {
                let v0 = face_vertices[tri[0] as usize].position;
                let v1 = face_vertices[tri[1] as usize].position;
                let v2 = face_vertices[tri[2] as usize].position;
                let n = compute_normal(v0, v1, v2);
                for &idx in tri {
                    let vert = &mut face_vertices[idx as usize];
                    vert.normal[0] += n[0];
                    vert.normal[1] += n[1];
                    vert.normal[2] += n[2];
                }
            }

            // Normalize normals
            for v in &mut face_vertices {
                let len = (v.normal[0].powi(2) + v.normal[1].powi(2) + v.normal[2].powi(2)).sqrt();
                if len > 1e-6 {
                    v.normal[0] /= len;
                    v.normal[1] /= len;
                    v.normal[2] /= len;
                }
            }

            face_indices = face_indices.iter().map(|e| e + index_offset).collect();
            index_offset += face_vertices.len() as u32;
            quad_id += 1;
            all_vertices.extend(face_vertices);
            all_indices.extend(face_indices);
            // Store one PrimitiveFace (shared vertices + indices)
        }
    }
    let mesh = Primitive {
        vertices: all_vertices,
        indices: all_indices,
    };
    (mesh, index_offset)
}
fn compute_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}
