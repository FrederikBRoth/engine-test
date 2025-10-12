use std::sync::Arc;

use sparmos_engine::{
    cgmath::*,
    core::{
        camera::{self, CameraController},
        state::{GameLoop, RenderContext, State, map_value},
    },
    entity::{
        entities::cube::mobius_strip,
        entity::{
            DrawMesh, Instance, Light, PrimitiveMesh, RenderableController, Rendering,
            instance_cube, instances_list_cylinder, make_cube_primitive, make_face_primitive,
            make_mesh_from_face,
        },
    },
    helpers::line_trace::{line_trace, line_trace_square},
    wgpu::{self, SurfaceConfiguration, naga::Range, wgc::device::queue},
    winit::{
        self,
        dpi::PhysicalPosition,
        event::{ElementState, KeyEvent, WindowEvent},
        keyboard::{Key, KeyCode},
    },
};

use crate::gameoflife::{InputType, Life};

pub struct MyLoop {
    pub score: u32,
    pub instance_controllers: Vec<RenderableController>,
    pub camera_controller: Option<CameraController>,
    pub counter: usize,
    pub life: Life,
    pub cursor_pos: PhysicalPosition<f32>,
    pub cursor_delta: (f64, f64),
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
        }
    }
}

impl GameLoop for MyLoop {
    fn render(
        &mut self,
        render: &mut wgpu::RenderPass,
        texture_view: &wgpu::TextureView,
        depth_texture: sparmos_engine::entity::texture::Texture,
    ) {
        for ics in self.instance_controllers.iter().as_ref() {
            if let Some(camera) = self.camera_controller.as_ref() {
                render.draw_meshes(
                    &ics,
                    &camera.camera_bind_group,
                    &ics.render_information.light_bind_group,
                );
            }
        }
    }

    fn update(&mut self, dt: std::time::Duration, rc: &RenderContext) {
        if let Some(camera_controller) = self.camera_controller.as_mut() {
            camera_controller.update_camera();
        }
        self.life.calculate_iteration(dt);
        if let Some(render) = self.instance_controllers.first_mut() {
            for (i, render_infos) in render.render_mesh_information.iter_mut().enumerate() {
                if self.life.game_area[i] == 1 {
                    render_infos
                        .instance_controller
                        .instances
                        .first_mut()
                        .unwrap()
                        .color = Vector3 {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
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
                    render_infos
                        .instance_controller
                        .instances
                        .first_mut()
                        .unwrap()
                        .color = color
                }
            }
        }
        for render in self.instance_controllers.iter_mut() {
            render
                .instance_manager
                .update_all(&rc.queue, &mut render.render_mesh_information);
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
                                    {
                                        if let Some(instance) =
                                            instance.instance_controller.instances.first_mut()
                                        {
                                            instance.color = Vector3 {
                                                x: 0.0,
                                                y: 1.0,
                                                z: 1.0,
                                            };
                                        }
                                    }
                                }
                                self.counter = self.counter + 1;
                            }
                        }
                        KeyCode::Space => {
                            if is_pressed {
                                self.life.toggle();
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
                                let test = camera_controller.camera.screen_to_world_ray(
                                    self.cursor_pos.x,
                                    self.cursor_pos.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                if let Some(controller) = self.instance_controllers.get_mut(0) {
                                    if let Some(i) = line_trace_square(controller, test) {
                                        self.life.game_area[i] = 1;
                                    }
                                }
                                self.life.toggle_input(None)
                            }
                        },

                        winit::event::MouseButton::Right => match state {
                            ElementState::Pressed => {
                                self.life.toggle_input(Some(InputType::Delete))
                            }
                            ElementState::Released => {
                                let test = camera_controller.camera.screen_to_world_ray(
                                    self.cursor_pos.x,
                                    self.cursor_pos.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                if let Some(controller) = self.instance_controllers.get_mut(0) {
                                    if let Some(i) = line_trace_square(controller, test) {
                                        self.life.game_area[i] = 0;
                                    }
                                }
                                self.life.toggle_input(None)
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
                        match input {
                            InputType::Insert => {
                                let test = camera_controller.camera.screen_to_world_ray(
                                    self.cursor_pos.x,
                                    self.cursor_pos.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                if let Some(controller) = self.instance_controllers.get_mut(0) {
                                    if let Some(i) = line_trace_square(controller, test) {
                                        self.life.game_area[i] = 1;
                                    }
                                }
                            }
                            InputType::Delete => {
                                let test = camera_controller.camera.screen_to_world_ray(
                                    self.cursor_pos.x,
                                    self.cursor_pos.y,
                                    screen.width as f32,
                                    screen.height as f32,
                                );
                                if let Some(controller) = self.instance_controllers.get_mut(0) {
                                    if let Some(i) = line_trace_square(controller, test) {
                                        self.life.game_area[i] = 0;
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
            0.4,
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
        let light_position = Vector3::new(-60.0, 20.0, 60.0);
        let light_source = Light::new(
            light_position,
            Vector3::new(1.0, 1.0, 1.0),
            &state.render_context.device,
        );

        let light_mesh = make_cube_primitive();
        let mut light_render = state.render_context.create_renderable_controller(
            vec![light_mesh],
            &light_source,
            &camera_controller,
            &primitive_shader,
            Some(vec![light_source.get_instance()]),
        );

        let radius = 2.0; // R: radius of the center circle
        let width = 0.4; // w: half-width of the strip
        let segments_u = 200; // num_u: subdivisions along the loop
        let segments_v = 20; // num_v: subdivisions across the strip

        let mobius_mesh = mobius_strip(radius, width, segments_u, segments_v, false);
        let mobius_mesh2 = mobius_strip(radius, width, segments_u, segments_v, true);

        let mut mobius_meshes: Vec<PrimitiveMesh> = mobius_mesh
            .iter()
            .map(|face| make_mesh_from_face(face))
            .collect();

        let mut mobius_meshes2: Vec<PrimitiveMesh> = mobius_mesh2
            .iter()
            .map(|face| make_mesh_from_face(face))
            .collect();

        for chunk in mobius_meshes2.chunks_mut(segments_v - 1) {
            chunk.reverse();
        }
        mobius_meshes.extend(mobius_meshes2);
        println!("{}", mobius_mesh.len());

        let mut instance_controller = state.render_context.create_renderable_controller(
            mobius_meshes,
            &light_source,
            &camera_controller,
            &primitive_shader,
            None,
        );

        instance_controller.instance_manager.update_all(
            &state.render_context.queue,
            &mut instance_controller.render_mesh_information,
        );
        let mut game_state = vec![0; (segments_v - 1) * (segments_u * 2)];
        game_state[30] = 1;
        game_state[31] = 1;
        game_state[32] = 1;
        game_state[33] = 1;

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
}
