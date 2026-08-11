use sparmos_engine::{
    application::state::{DeviceBackend, Game, State, map_value},
    cgmath::{self, *},
    core::{
        buffer::{Buffer, BufferType, UniformParameters},
        engine::Engine,
        entities::World,
        geometry::Primitive,
        instance::{Instance, InstanceController, InstanceRaw},
        material::MaterialBuilder,
        render::Renderable,
        texture::Texture,
    },
    egui::{self, Color32, Rect, Response, Sense, Ui, Vec2},
    entities::cube,
    helpers::line_trace::line_trace_square,
    log,
    systems::{
        camera::{Camera, CameraSystem},
        light::{Light, LightSystem},
    },
    web_time,
    wgpu::{self},
    winit::{
        self,
        dpi::{PhysicalPosition, PhysicalSize},
        event::{ElementState, KeyEvent, WindowEvent},
        keyboard::KeyCode,
    },
};

use crate::{
    gameoflife::{
        InputType, Life, Lifeform, MobiusSize, double_sided_mobius_strip, get_neighbor_indices,
    },
    markers::{self, Mobius},
};

pub struct MobiusVisualizer {
    pub score: u32,
    pub counter: usize,
    pub life: Life,
    pub cursor_pos: PhysicalPosition<f32>,
    pub cursor_delta: (f64, f64),
    pub gui_state: GuiState,
}

pub struct GuiState {
    lifeform_toggled: bool,
    mobius_toggled: bool,
    new_lifeform_width: f32,
    new_lifeform_height: f32,
    prev_width: f32,
    prev_height: f32,
    current_lifeform: Option<Lifeform>,
    selected_lifeform: Option<Lifeform>,
    side_bar_max: f32,
    side_bar_min: f32,
    mobius_radius: f32,
    mobius_width: f32,
    mobius_grid_count_width: usize,
    mobius_grid_count_height: usize,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            lifeform_toggled: false,
            mobius_toggled: false,
            new_lifeform_height: 7.0,
            new_lifeform_width: 7.0,
            prev_width: 0.0,
            prev_height: 0.0,
            current_lifeform: None,
            selected_lifeform: None,
            side_bar_max: 250.0,
            side_bar_min: 250.0,
            mobius_radius: 2.0,
            mobius_width: 2.0,
            mobius_grid_count_height: 300,
            mobius_grid_count_width: 30,
        }
    }
}
impl Default for MobiusVisualizer {
    fn default() -> Self {
        Self {
            score: 0,
            counter: 0,
            life: Life::new(vec![], 0, 0, 4.0),
            cursor_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            cursor_delta: (0.0, 0.0),
            gui_state: GuiState::default(),
        }
    }
}

impl Game for MobiusVisualizer {
    fn update(&mut self, dt: std::time::Duration, engine: &mut Engine, world: &mut World) {
        // let mut camera_system = self.world.query::<&mut CameraSystem>();
        // let camera_system = camera_system.iter().next().unwrap();
        let mut query = world.entities.query::<&mut Camera>();
        let camera = query.iter().next().expect("No camera found");
        let camera_system = world.resources.get_system_mut::<CameraSystem>().unwrap();
        camera_system.update_camera(dt, &engine.render_context, camera);

        self.life.calculate_iteration(dt);

        for (_, material) in engine.render_context.gpu_objects.materials.iter_mut() {
            if let Some(texture) = material.texture.as_mut() {
                self.life
                    .upload_to_texture(&engine.render_context.queue, &texture.texture);
            }
        }
        for render in engine
            .render_context
            .gpu_objects
            .instance_controllers
            .iter_mut()
        {
            render.1.update(&engine.render_context.queue);
        }
    }

    fn process_event(
        &mut self,
        event: &winit::event::WindowEvent,
        screen: &winit::dpi::PhysicalSize<u32>,
        engine: &mut Engine,
        world: &mut World,
    ) {
        // let mut camera_system = self.world.query::<&mut CameraSystem>();
        // let camera_system = camera_system.iter().next().unwrap();
        // let (entity, camera) = state
        let mut query = world.entities.query::<&mut Camera>();
        let camera = query.iter().next().expect("No camera found");
        let camera_system = world.resources.get_system_mut::<CameraSystem>().unwrap();
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: winit::keyboard::PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => {
                let var_name = *state == ElementState::Pressed;
                let is_pressed = var_name;
                match keycode {
                    // KeyCode::Delete => {
                    //     if let Some(instance_controller) = self.instance_controllers.get_mut(0) {
                    //         for n in self.counter..self.counter + 1 {
                    //             if let Some(instance) =
                    //                 instance_controller.render_mesh_information.get_mut(n)
                    //                 && let Some(instance) =
                    //                     instance.instance_controller.instances.first_mut()
                    //             {
                    //                 instance.color = Vector3 {
                    //                     x: 1.0,
                    //                     y: 1.0,
                    //                     z: 1.0,
                    //                 };
                    //             }
                    //         }
                    //         self.counter += 1;
                    //     }
                    // }
                    KeyCode::Space => {
                        if is_pressed {
                            self.life.toggle();
                            println!("Life Toggled!")
                        }
                        #[cfg(target_arch = "wasm32")]
                        log::warn!("spaced pressed")
                    }
                    // KeyCode::KeyG => {
                    //     if is_pressed {
                    //         let click_ray = camera_controller.camera.screen_to_world_ray(
                    //             self.cursor_pos.x,
                    //             self.cursor_pos.y,
                    //             screen.width as f32,
                    //             screen.height as f32,
                    //         );
                    //         if let Some(controller) = self.instance_controllers.get_mut(0) {
                    //             for (i, render_infos) in
                    //                 controller.render_mesh_information.iter_mut().enumerate()
                    //             {
                    //                 let bounds = Vector2 {
                    //                     x: render_infos.vertex_offset,
                    //                     y: render_infos.vertex_offset + render_infos.num_vertices,
                    //                 };
                    //
                    //                 if let Some(i) = line_trace_square(
                    //                     &controller.vertices,
                    //                     bounds,
                    //                     click_ray,
                    //                     None,
                    //                 ) && let Some(lifeform) =
                    //                     self.gui_state.selected_lifeform.clone()
                    //                 {
                    //                     self.life.insert_premade_lifeform(lifeform, i);
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }
                    _ => (),
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                match button {
                    winit::event::MouseButton::Left => match state {
                        ElementState::Pressed => self.life.toggle_input(Some(InputType::Insert)),
                        ElementState::Released => {
                            let click_ray = camera.screen_to_world_ray(
                                self.cursor_pos.x,
                                self.cursor_pos.y,
                                screen.width as f32,
                                screen.height as f32,
                            );
                            let mut query = world.entities.query::<(&Renderable, &Primitive)>();
                            let (renderable, primitive) =
                                query.iter().next().expect("No camera found");

                            let start = web_time::Instant::now();
                            if let Some(hit) =
                                line_trace_square(&primitive.vertices, click_ray, None)
                            {
                                self.life.game_area[hit as usize] = 1;
                            }

                            self.life.toggle_input(None);
                            self.life.prev_drawed_elem = None;
                        }
                    },

                    winit::event::MouseButton::Right => match state {
                        ElementState::Pressed => self.life.toggle_input(Some(InputType::Delete)),
                        ElementState::Released => {
                            let click_ray = camera.screen_to_world_ray(
                                self.cursor_pos.x,
                                self.cursor_pos.y,
                                screen.width as f32,
                                screen.height as f32,
                            );

                            // if let Some(controller) = self.instance_controllers.get_mut(0) {
                            //     for (i, render_infos) in
                            //         controller.render_mesh_information.iter_mut().enumerate()
                            //     {
                            //         let bounds = Vector2 {
                            //             x: render_infos.vertex_offset,
                            //             y: render_infos.vertex_offset + render_infos.num_vertices,
                            //         };
                            //
                            //         if let Some(i) = line_trace_square(
                            //             &controller.vertices,
                            //             bounds,
                            //             click_ray,
                            //             None,
                            //         ) {
                            //             self.life.game_area[i] = 0;
                            //         }
                            //     }
                            // }
                            // self.life.toggle_input(None);
                            // self.life.prev_drawed_elem = None;
                        }
                    },

                    // winit::event::MouseButton::Right => todo!(),
                    // winit::event::MouseButton::Middle => todo!(),
                    // winit::event::MouseButton::Back => todo!(),
                    // winit::event::MouseButton::Forward => todo!(),
                    // winit::event::MouseButton::Other(_) => todo!(),
                    _ => {}
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = PhysicalPosition::new(position.x as f32, position.y as f32);

                let mut query = world.entities.query::<(&Renderable, &Primitive)>();
                let (renderable, primitive) = query.iter().next().expect("No camera found");
                if !self.life.enabled {
                    camera_system.process_mouse(
                        self.cursor_delta.0 as f32,
                        -self.cursor_delta.1 as f32,
                        camera,
                    );
                }
                if let Some(input) = &self.life.input {
                    let data = match input {
                        InputType::Insert => 1,
                        InputType::Delete => 0,
                    };
                    let click_ray = camera.screen_to_world_ray(
                        self.cursor_pos.x,
                        self.cursor_pos.y,
                        screen.width as f32,
                        screen.height as f32,
                    );
                    if let Some(prev_elem) = self.life.prev_drawed_elem {
                        let indices =
                            get_neighbor_indices(prev_elem, self.life.width, self.life.height, 10);

                        let start = web_time::Instant::now();
                        if let Some(hit) =
                            line_trace_square(&primitive.vertices, click_ray, Some(indices))
                        {
                            self.life.game_area[hit as usize] = 1;
                            self.life.prev_drawed_elem = Some(hit as usize);
                        }
                    } else if let Some(hit) =
                        line_trace_square(&primitive.vertices, click_ray, None)
                    {
                        self.life.game_area[hit as usize] = 1;
                        self.life.prev_drawed_elem = Some(hit as usize);
                    }
                }

                // let test = self.camera_controller.camera.screen_to_world_ray(
                //     self.cursor_position.x,
                //     self.cursor_position.y,
                //     screen.width as f32,
                //     screen.height as f32,
                // );
                // line_trace(&mut self.instance_controller2, camera, &self.queue, &self.device, test);

                // if let Some(controller) = self.chunk_map.get_mut(&target_chunk) {
                //     if let Some(i) = line_trace(controller, test) {
                //         controller.remove_instance(i, &self.queue);
                //     }
                // }
            }

            _ => (),
        }
        camera_system.process_events(event, camera);
    }

    fn setup(&mut self, state: &mut State) {
        let engine = &mut state.engine;
        let world = &mut state.world;

        let camera = Camera::new(PhysicalSize::new(
            state.size.width as f32,
            state.size.height as f32,
        ));
        let camera_system = CameraSystem::new(75.0, 50.0, &engine.render_context.device, &camera);
        //registers system and creates bind_group

        let light = Light {
            position: cgmath::vec3(100.0, 100.0, 1.0),
            color: cgmath::vec3(1.0, 0.0, 0.0),
        };

        let light2 = Light {
            position: cgmath::vec3(-100.0, -100.0, 1.0),
            color: cgmath::vec3(0.0, 1.0, 0.0),
        };
        let light_system = LightSystem::init(
            &[light.clone(), light2.clone()],
            &engine.render_context.device,
        );
        world.add_entity((camera,));
        world.add_system(camera_system);
        world.add_system(light_system);
        let primitive_shader =
            engine
                .render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("lights"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shaders/lights.wgsl").into()),
                });
        let mobius_shader =
            engine
                .render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("mobius"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mobius.wgsl").into()),
                });

        engine
            .render_context
            .shaders
            .insert("lights".to_string(), primitive_shader);
        engine
            .render_context
            .shaders
            .insert("mobius".to_string(), mobius_shader);

        let radius = 2.0; // R: radius of the center circle
        let width = 2.0; // w: half-width of the strip
        let segments_u = 600; // num_u: subdivisions along the loop
        let segments_v = 100; // num_v: subdivisions across the strip

        let start = web_time::Instant::now();
        let mobius_mesh = double_sided_mobius_strip(radius, width, segments_u, segments_v);

        // let mobius_mesh = cube::new();
        let grid_width = (segments_v - 1) as u32;
        let grid_height = (segments_u * 2) as u32;

        let life_texture = Texture::create_life_texture(
            &engine.render_context.device,
            &engine.render_context.queue,
            grid_width,
            grid_height,
            "GameOfLifeTexture",
        );
        let mesh = mobius_mesh.make_mb(&mut engine.render_context);

        let ic = InstanceController::<InstanceRaw>::new(
            vec![Instance::default()],
            &mut engine.render_context,
        );

        let duration = start.elapsed();
        println!("Time elapsed: {:?}", duration);

        let mobius_size = MobiusSize {
            width: grid_width,
            height: grid_height,
        };

        let mobius_size_buffer = Buffer::new_init(
            &[mobius_size],
            &engine.render_context.device,
            BufferType::UniformBuffer(UniformParameters::default()),
        );
        let mobius_mat = MaterialBuilder::new()
            .add_layout(
                "camera",
                world.resources.get_system::<CameraSystem>().unwrap(),
            )
            .add_layout(
                "light",
                world.resources.get_system::<LightSystem>().unwrap(),
            )
            .add_texture(life_texture)
            .add_shader("mobius")
            .add_buffer(0, mobius_size_buffer)
            .build(&mesh, &ic, &mut engine.render_context);

        let cube_mesh = cube::new().make_mb(&mut engine.render_context);
        let light_ic = InstanceController::<InstanceRaw>::new(
            vec![
                Instance::new([100.0, 100.0, 1.0].into(), 1.0),
                Instance::new([-100.0, -100.0, 1.0].into(), 1.0),
            ],
            &mut engine.render_context,
        );
        let light_mat = MaterialBuilder::new()
            .add_layout(
                "camera",
                world.resources.get_system::<CameraSystem>().unwrap(),
            )
            .add_layout(
                "light",
                world.resources.get_system::<LightSystem>().unwrap(),
            )
            .add_shader("lights")
            .build(&cube_mesh, &light_ic, &mut engine.render_context);

        //Should group renderList by material, so that we dont do many pipeline shifts

        let light_entity = Renderable {
            material_handle: light_mat,
            instance_controller_handle: light_ic,
            mesh_handle: cube_mesh,
        };

        let mobius_entity = Renderable {
            material_handle: mobius_mat,
            mesh_handle: mesh,
            instance_controller_handle: ic,
        };

        world.add_entity((light_entity, markers::Light));
        world.add_entity((mobius_entity, mobius_mesh, markers::Mobius));

        let game_state = vec![0; (segments_v - 1) * (segments_u * 2)];

        self.life = Life::new(
            game_state,
            (segments_v - 1) as i32,
            (segments_u * 2) as i32,
            2.0,
        );
    }

    fn resize(&mut self, engine: &mut Engine, world: &mut World) {
        // let mut camera_system = self.world.query::<&mut CameraSystem>o();
        // let camera_system = camera_system.iter().next().unwrap();

        let mut query = world.entities.query::<&mut Camera>();
        let camera = query.iter().next().expect("No camera found");
        let camera_system = world.resources.get_system_mut::<CameraSystem>();

        camera.aspect =
            engine.render_context.config.width as f32 / engine.render_context.config.height as f32;
        println!("{:?}", camera.aspect);
        let new_fov = map_value(camera.aspect, 0.8, 1.88, 25.0, 55.0);
        camera.fovy = new_fov;
        // if camera.aspect < camera.camera_animator.aspect_ratio_limit {
        //     let eye = Point3::new(110.0, 90.0, -130.0);
        //     let target = Point3::new(20.0, 25.0, 20.0);
        //     camera.eye = eye;
        //     camera.target = target;
        //     camera.fovy = 90.0;
        // }
    }

    fn gui_setup(&mut self, dt: std::time::Duration, engine: &mut Engine, ui: &mut Ui) {
        egui::Panel::top("my_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .toggle_value(&mut self.gui_state.lifeform_toggled, "Lifeform Library")
                    .clicked()
                {
                    self.gui_state.mobius_toggled = false;
                };
                if ui
                    .toggle_value(
                        &mut self.gui_state.mobius_toggled,
                        "Mobius Strip Parameters",
                    )
                    .clicked()
                {
                    self.gui_state.lifeform_toggled = false;
                };
            });
        });

        if self.gui_state.mobius_toggled {
            egui::Panel::left("mobius_panel")
                .resizable(false)
                .min_size(self.gui_state.side_bar_min)
                .max_size(self.gui_state.side_bar_max)
                .show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.vertical_centered(|ui| {
                        ui.heading("Mobius Parameters");
                        ui.separator();
                        let grid_height = ui.add(
                            egui::Slider::new(
                                &mut self.gui_state.mobius_grid_count_height,
                                100..=600,
                            )
                            .text("Height grids"),
                        );
                        let grid_width = ui.add(
                            egui::Slider::new(
                                &mut self.gui_state.mobius_grid_count_width,
                                10..=100,
                            )
                            .text("Width grids"),
                        );

                        let radius = ui.add(
                            egui::Slider::new(&mut self.gui_state.mobius_radius, 0.5..=40.0)
                                .text("Radius"),
                        );
                        let width = ui.add(
                            egui::Slider::new(&mut self.gui_state.mobius_width, 0.5..=40.0)
                                .text("Width"),
                        );

                        if grid_height.changed()
                            || grid_width.changed()
                            || radius.changed()
                            || width.changed()
                        {
                            let new_mobius = double_sided_mobius_strip(
                                self.gui_state.mobius_radius,
                                self.gui_state.mobius_width,
                                self.gui_state.mobius_grid_count_height,
                                self.gui_state.mobius_grid_count_width,
                            );
                            // self.instance_controllers
                            //     .first_mut()
                            //     .unwrap()
                            //     .update_mesh_data(vec![new_mobius], &render_context.device);
                            let game_state = vec![
                                0;
                                (self.gui_state.mobius_grid_count_width - 1)
                                    * (self.gui_state.mobius_grid_count_height
                                        * 2)
                            ];

                            self.life = Life::new(
                                game_state,
                                (self.gui_state.mobius_grid_count_width - 1) as i32,
                                (self.gui_state.mobius_grid_count_height * 2) as i32,
                                2.0,
                            );
                        }

                        // Do something when width changes
                    });
                });
        }
        if self.gui_state.lifeform_toggled {
            egui::Panel::left("backend_panel")
                .resizable(false)
                .min_size(self.gui_state.side_bar_min)
                .max_size(self.gui_state.side_bar_max)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2]) // Optional: prevent auto-shrinking
                        .show(ui, |ui| {
                            ui.add_space(4.0);
                            ui.vertical_centered(|ui| {
                                ui.heading("Library");
                            });
                            ui.separator();

                            ui.collapsing("New Lifeform", |ui| {
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.gui_state.new_lifeform_width,
                                        0.0..=30.0,
                                    )
                                    .step_by(1.0)
                                    .text("Width"),
                                );
                                ui.add(
                                    egui::Slider::new(
                                        &mut self.gui_state.new_lifeform_height,
                                        0.0..=30.0,
                                    )
                                    .step_by(1.0)
                                    .text("Height"),
                                );

                                if self.gui_state.new_lifeform_height != self.gui_state.prev_height
                                    || self.gui_state.new_lifeform_width
                                        != self.gui_state.prev_width
                                {
                                    self.gui_state.current_lifeform = Some(Lifeform::new(
                                        format!("New Lifeform {}", self.life.lifeforms.len()),
                                        self.gui_state.new_lifeform_width as u32,
                                        self.gui_state.new_lifeform_height as u32,
                                    ));
                                    self.gui_state.prev_width = self.gui_state.new_lifeform_width;
                                    self.gui_state.prev_height = self.gui_state.new_lifeform_height;
                                }

                                if let Some(lifeform) = &mut self.gui_state.current_lifeform {
                                    self.gui_state.side_bar_max = if 25.0
                                        * (lifeform.width + 7) as f32
                                        >= self.gui_state.side_bar_max
                                    {
                                        25.0 * (lifeform.width + 7) as f32
                                    } else if 25.0 * (lifeform.width as f32)
                                        < self.gui_state.side_bar_max
                                        && 25.0 * ((lifeform.width + 2) as f32)
                                            > self.gui_state.side_bar_min
                                    {
                                        25.0 * (lifeform.width + 2) as f32
                                    } else {
                                        self.gui_state.side_bar_min
                                    };
                                    editable_lifeform_gui(lifeform, ui);
                                    if ui.button("create").clicked() {
                                        self.life.add_lifeform(lifeform.clone());
                                    }
                                }
                            });
                            ui.separator();

                            for lifeform in self.life.lifeforms.iter() {
                                if let Some(res) = static_lifeform_gui(lifeform, ui) {
                                    if res.clicked() {
                                        self.gui_state.selected_lifeform = Some(lifeform.clone());
                                        println!("Selected lifeform: {}", lifeform.name)
                                    }
                                }
                            }
                        });
                });
        }
    }
}
fn editable_lifeform_gui(lifeform: &mut Lifeform, ui: &mut Ui) {
    ui.text_edit_singleline(&mut lifeform.name);
    egui::Frame::new()
        .inner_margin(6)
        .outer_margin(6)
        .corner_radius(10.0)
        .fill(Color32::GRAY)
        .show(ui, |ui| {
            let grid_size = 25.0;

            let start_pos = ui.cursor().min;

            for y in 0..lifeform.height {
                for x in 0..lifeform.width {
                    let idx = y * lifeform.width + x;

                    let pos = start_pos
                        + Vec2 {
                            x: x as f32 * grid_size,
                            y: y as f32 * grid_size,
                        };
                    let rect = Rect::from_min_size(
                        pos,
                        Vec2 {
                            x: grid_size,
                            y: grid_size,
                        },
                    );

                    let response: Response = ui.interact(
                        rect,
                        ui.id().with((x, y, lifeform.name.clone())),
                        Sense::click_and_drag(),
                    );

                    if response.clicked() {
                        lifeform.data[idx as usize] ^= 1; // Toggle cell
                    }

                    let color = if lifeform.data[idx as usize] == 1 {
                        Color32::WHITE
                    } else {
                        Color32::BLACK
                    };
                    ui.painter().rect_filled(rect, 0.0, color);
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        (0.1, Color32::GRAY),
                        egui::StrokeKind::Inside,
                    );

                    if response.hovered() {
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            (0.5, Color32::WHITE),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }

            // Reserve layout space
            ui.allocate_exact_size(
                Vec2 {
                    x: lifeform.width as f32 * grid_size,
                    y: lifeform.height as f32 * grid_size,
                },
                Sense::hover(),
            );
        });
}

fn static_lifeform_gui(lifeform: &Lifeform, ui: &mut Ui) -> Option<Response> {
    ui.label(lifeform.name.clone());
    let mut response = None;
    egui::Frame::new()
        .inner_margin(6)
        .outer_margin(6)
        .corner_radius(10)
        .fill(egui::Color32::GRAY)
        .show(ui, |ui| {
            let available_width = ui.available_width();
            let grid_size_x = available_width / lifeform.width as f32;
            let grid_size_y = available_width / lifeform.height as f32;

            let grid_size = if grid_size_x > grid_size_y {
                grid_size_y
            } else {
                grid_size_x
            };
            let start_pos = ui.cursor().min;
            for y in 0..lifeform.height {
                for x in 0..lifeform.width {
                    let idx = y * lifeform.width + x;
                    let pos = start_pos + egui::vec2(x as f32 * grid_size, y as f32 * grid_size);
                    let rect = egui::Rect::from_min_size(pos, egui::vec2(grid_size, grid_size));
                    let color = if lifeform.data[idx as usize] == 1 {
                        Color32::WHITE
                    } else {
                        Color32::BLACK
                    };
                    ui.painter().rect_filled(rect, 0.0, color);
                }
            }
            let (_, res) = ui.allocate_exact_size(
                egui::vec2(
                    lifeform.width as f32 * grid_size,
                    lifeform.height as f32 * grid_size,
                ),
                Sense::click_and_drag(),
            );
            response = Some(res);
        });
    response
}
