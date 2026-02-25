use std::{sync::Arc, time::Instant};

use sparmos_engine::{
    cgmath::*,
    core::{
        camera::CameraController,
        state::{DeviceBackend, GameLoop, RenderContext, Renderable, State, map_value},
    },
    egui::{self, Color32, Rect, Response, Sense, Ui, Vec2, response},
    entity::entity::{
        Color, DrawMesh, InstanceController, InstanceStorage, Light, PrimitiveMesh,
        RenderMeshInformation, RenderableController, instance_cube, make_cube_primitive,
    },
    helpers::line_trace::line_trace_square,
    web_time,
    wgpu::{self, SurfaceConfiguration, wgc::device},
    winit::{
        self,
        dpi::PhysicalPosition,
        event::{ElementState, KeyEvent, WindowEvent},
        keyboard::KeyCode,
    },
};

use crate::gameoflife::{
    InputType, Life, Lifeform, double_sided_mobius_strip, get_neighbor_indices, mobius_strip,
};

pub struct MyLoop {
    pub score: u32,
    pub instance_controllers: Vec<RenderableController>,
    pub camera_controller: Option<CameraController>,
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
impl Default for MyLoop {
    fn default() -> Self {
        Self {
            score: 0,
            instance_controllers: vec![],
            camera_controller: None,
            counter: 0,
            life: Life::new(vec![], 0, 0, 4.0),
            cursor_pos: PhysicalPosition { x: 0.0, y: 0.0 },
            cursor_delta: (0.0, 0.0),
            gui_state: GuiState::default(),
        }
    }
}

impl GameLoop for MyLoop {
    fn render(
        &mut self,
        render: &mut wgpu::RenderPass,
        _texture_view: &wgpu::TextureView,
        _depth_texture: sparmos_engine::entity::texture::Texture,
        backend: &DeviceBackend,
    ) {
        for ics in self.instance_controllers.iter().as_ref() {
            if let Some(camera) = self.camera_controller.as_ref() {
                render.draw_meshes(
                    ics,
                    &camera.camera_bind_group,
                    &ics.render_information.light_bind_group,
                    backend,
                );
            }
        }
    }

    fn update(&mut self, dt: std::time::Duration, rc: &RenderContext) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.update_camera(dt);
        }
        self.life.calculate_iteration(dt);
        if let Some(render) = self.instance_controllers.first_mut() {
            for (i, render_infos) in render.render_mesh_information.iter_mut().enumerate() {
                let end = (render_infos.num_vertices - render_infos.vertex_offset) / 4;

                for i in 0..end as usize {
                    if self.life.game_area[i] == 1 {
                        if let Some(storage) = render.storage_buffer.as_mut() {
                            storage.instances.get_mut(i).unwrap().color = [1.0, 0.0, 0.0];
                        }
                    } else {
                        let color = if self.life.enabled {
                            Vector3 {
                                x: 1.0,
                                y: 1.0,
                                z: 1.0,
                            }
                        } else {
                            Vector3 {
                                x: 0.4,
                                y: 0.4,
                                z: 0.4,
                            }
                        };
                        if let Some(storage) = render.storage_buffer.as_mut() {
                            storage.instances.get_mut(i).unwrap().color = color.into();
                        }
                    }
                }
            }
        }
        for render in self.instance_controllers.iter_mut() {
            render.update_all(&rc.queue);
        }
    }

    fn process_event(
        &mut self,
        event: &winit::event::WindowEvent,
        screen: &winit::dpi::PhysicalSize<u32>,
    ) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
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
                        KeyCode::Delete => {
                            if let Some(instance_controller) = self.instance_controllers.get_mut(0)
                            {
                                for n in self.counter..self.counter + 1 {
                                    if let Some(instance) =
                                        instance_controller.render_mesh_information.get_mut(n)
                                        && let Some(instance) =
                                            instance.instance_controller.instances.first_mut()
                                    {
                                        instance.color = Vector3 {
                                            x: 1.0,
                                            y: 1.0,
                                            z: 1.0,
                                        };
                                    }
                                }
                                self.counter += 1;
                            }
                        }
                        KeyCode::Space => {
                            if is_pressed {
                                self.life.toggle();
                            }
                        }
                        KeyCode::KeyG => {
                            if is_pressed {
                                let click_ray = camera_controller.camera.screen_to_world_ray(
                                    self.cursor_pos.x,
                                    self.cursor_pos.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                if let Some(controller) = self.instance_controllers.get_mut(0) {
                                    for (i, render_infos) in
                                        controller.render_mesh_information.iter_mut().enumerate()
                                    {
                                        let bounds = Vector2 {
                                            x: render_infos.vertex_offset,
                                            y: render_infos.vertex_offset
                                                + render_infos.num_vertices,
                                        };

                                        if let Some(i) = line_trace_square(
                                            &controller.vertices,
                                            bounds,
                                            click_ray,
                                            None,
                                        ) && let Some(lifeform) =
                                            self.gui_state.selected_lifeform.clone()
                                        {
                                            self.life.insert_premade_lifeform(lifeform, i);
                                        }
                                    }
                                }
                            }
                        }
                        _ => (),
                    }
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    match button {
                        winit::event::MouseButton::Left => match state {
                            ElementState::Pressed => {
                                self.life.toggle_input(Some(InputType::Insert))
                            }
                            ElementState::Released => {
                                let click_ray = camera_controller.camera.screen_to_world_ray(
                                    self.cursor_pos.x,
                                    self.cursor_pos.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                if let Some(controller) = self.instance_controllers.get_mut(0) {
                                    for (i, render_infos) in
                                        controller.render_mesh_information.iter_mut().enumerate()
                                    {
                                        let bounds = Vector2 {
                                            x: render_infos.vertex_offset,
                                            y: render_infos.vertex_offset
                                                + render_infos.num_vertices,
                                        };

                                        if let Some(i) = line_trace_square(
                                            &controller.vertices,
                                            bounds,
                                            click_ray,
                                            None,
                                        ) {
                                            self.life.game_area[i] = 1;
                                        }
                                    }
                                }

                                self.life.toggle_input(None);
                                self.life.prev_drawed_elem = None;
                            }
                        },

                        winit::event::MouseButton::Right => match state {
                            ElementState::Pressed => {
                                self.life.toggle_input(Some(InputType::Delete))
                            }
                            ElementState::Released => {
                                let click_ray = camera_controller.camera.screen_to_world_ray(
                                    self.cursor_pos.x,
                                    self.cursor_pos.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                if let Some(controller) = self.instance_controllers.get_mut(0) {
                                    for (i, render_infos) in
                                        controller.render_mesh_information.iter_mut().enumerate()
                                    {
                                        let bounds = Vector2 {
                                            x: render_infos.vertex_offset,
                                            y: render_infos.vertex_offset
                                                + render_infos.num_vertices,
                                        };

                                        if let Some(i) = line_trace_square(
                                            &controller.vertices,
                                            bounds,
                                            click_ray,
                                            None,
                                        ) {
                                            self.life.game_area[i] = 0;
                                        }
                                    }
                                }
                                self.life.toggle_input(None);
                                self.life.prev_drawed_elem = None;
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

                    if !self.life.enabled {
                        camera_controller
                            .process_mouse(self.cursor_delta.0 as f32, -self.cursor_delta.1 as f32);
                    }
                    if let Some(input) = &self.life.input {
                        let data = match input {
                            InputType::Insert => 1,
                            InputType::Delete => 0,
                        };
                        let click_ray = camera_controller.camera.screen_to_world_ray(
                            self.cursor_pos.x,
                            self.cursor_pos.y,
                            screen.width as f32,
                            screen.height as f32,
                        );
                        if let Some(controller) = self.instance_controllers.get_mut(0) {
                            if let Some(prev_elem) = self.life.prev_drawed_elem {
                                let indices = get_neighbor_indices(
                                    prev_elem,
                                    self.life.width,
                                    self.life.height,
                                    10,
                                );

                                for (i, render_infos) in
                                    controller.render_mesh_information.iter_mut().enumerate()
                                {
                                    let bounds = Vector2 {
                                        x: render_infos.vertex_offset,
                                        y: render_infos.vertex_offset + render_infos.num_vertices,
                                    };

                                    if let Some(i) = line_trace_square(
                                        &controller.vertices,
                                        bounds,
                                        click_ray,
                                        Some(indices.clone()),
                                    ) {
                                        self.life.game_area[i] = data;
                                        self.life.prev_drawed_elem = Some(i);
                                    }
                                }
                            } else {
                                for (i, render_infos) in
                                    controller.render_mesh_information.iter_mut().enumerate()
                                {
                                    let bounds = Vector2 {
                                        x: render_infos.vertex_offset,
                                        y: render_infos.vertex_offset + render_infos.num_vertices,
                                    };

                                    if let Some(i) = line_trace_square(
                                        &controller.vertices,
                                        bounds,
                                        click_ray,
                                        None,
                                    ) {
                                        self.life.game_area[i] = data;
                                        self.life.prev_drawed_elem = Some(i);
                                    }
                                }
                            }
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
            camera_controller.process_events(event);
        }
    }

    fn setup<S: GameLoop>(&mut self, state: &mut State<S>) {
        let camera_controller = CameraController::new(
            75.0,
            50.0,
            state.size,
            &state.render_context.device,
            Arc::clone(&state.render_context.queue),
        );
        let primitive_shader =
            state
                .render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("PrimitiveShader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shaders/primitive.wgsl").into()),
                });
        let mobius_shader =
            state
                .render_context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("PrimitiveShader"),
                    source: wgpu::ShaderSource::Wgsl(include_str!("shaders/mobius.wgsl").into()),
                });
        let light_position = Vector3::new(-60.0, 20.0, 60.0);
        let light_source = Light::new(
            light_position,
            Vector3::new(1.0, 1.0, 1.0),
            &state.render_context.device,
        );

        let light_mesh = make_cube_primitive();

        let light_renderable = Renderable {
            mesh: light_mesh,
            ic: InstanceController::new(vec![light_source.get_instance()]),
        };

        let light_render = state.render_context.create_renderable_controller(
            vec![light_renderable],
            &light_source,
            &camera_controller,
            &primitive_shader,
            None,
        );

        let radius = 2.0; // R: radius of the center circle
        let width = 2.0; // w: half-width of the strip
        let segments_u = 600; // num_u: subdivisions along the loop
        let segments_v = 100; // num_v: subdivisions across the strip

        let start = web_time::Instant::now();
        let mobius_mesh = double_sided_mobius_strip(radius, width, segments_u, segments_v);

        let mut mobius_instance_list = vec![];

        for _ in 0..(mobius_mesh.vertices.len() / 4) {
            mobius_instance_list.push(Color {
                color: [0.0, 1.0, 1.0],
                _pad: 0.0,
            });
        }
        println!("Size of Color: {}", std::mem::size_of::<Color>());
        println!("{:?}", mobius_instance_list.len());

        let mobius_storage =
            InstanceStorage::new(mobius_instance_list, &state.render_context.device);

        let mobius_renderable = Renderable {
            mesh: mobius_mesh,
            ic: InstanceController::new(vec![instance_cube(
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 1.0,
                },
            )]),
        };
        let duration = start.elapsed();
        println!("Time elapsed: {:?}", duration);

        let mut instance_controller = state.render_context.create_renderable_controller(
            vec![mobius_renderable],
            &light_source,
            &camera_controller,
            &mobius_shader,
            Some(mobius_storage),
        );

        instance_controller.update_all(&state.render_context.queue);

        let mut game_state = vec![0; (segments_v - 1) * (segments_u * 2)];

        game_state[30] = 1;
        game_state[31] = 1;
        game_state[32] = 1;

        self.life = Life::new(
            game_state,
            (segments_v - 1) as i32,
            (segments_u * 2) as i32,
            2.0,
        );

        self.instance_controllers.push(instance_controller);
        self.instance_controllers.push(light_render);

        self.camera_controller = Some(camera_controller);
    }

    fn resize(&mut self, config: &SurfaceConfiguration) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.camera.aspect = config.width as f32 / config.height as f32;
            println!("{:?}", camera_controller.camera.aspect);
            let new_fov = map_value(camera_controller.camera.aspect, 0.8, 1.88, 25.0, 55.0);
            camera_controller.camera.fovy = new_fov;
            if camera_controller.camera.aspect
                < camera_controller.camera.camera_animator.aspect_ratio_limit
            {
                let eye = Point3::new(110.0, 90.0, -130.0);
                let target = Point3::new(20.0, 25.0, 20.0);
                camera_controller.camera.eye = eye;
                camera_controller.camera.target = target;
                camera_controller.camera.fovy = 90.0;
            }
        }
    }

    fn gui_setup(
        &mut self,
        egui_renderer: &sparmos_engine::core::gui::EguiRenderer,
        render_context: &RenderContext,
    ) {
        egui::TopBottomPanel::top("my_panel").show(egui_renderer.context(), |ui| {
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
            egui::SidePanel::left("mobius_panel")
                .resizable(false)
                .min_width(self.gui_state.side_bar_min)
                .max_width(self.gui_state.side_bar_max)
                .show(egui_renderer.context(), |ui| {
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
                            self.instance_controllers
                                .first_mut()
                                .unwrap()
                                .update_mesh_data(vec![new_mobius], &render_context.device);
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
            egui::SidePanel::left("backend_panel")
                .resizable(false)
                .min_width(self.gui_state.side_bar_min)
                .max_width(self.gui_state.side_bar_max)
                .show(egui_renderer.context(), |ui| {
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
