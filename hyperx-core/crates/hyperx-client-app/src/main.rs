use clap::Parser;
use glam::{DVec3, Vec3};
use hyperx_core_render::camera::Camera;
use hyperx_core_render::mesh::mesh_section;
use hyperx_core_render::pipeline::RenderContext;
use hyperx_core_world::chunk::{ChunkColumn, MIN_Y};
use hyperx_core_world::physics::Aabb;
use hyperx_core_world::World;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

#[derive(Parser, Debug)]
#[command(name = "hyperx-core")]
#[command(about = "HyperXLauncher Native Rust Minecraft Engine Core", version = "0.1.0")]
pub struct CliArgs {
    /// Player display username
    #[arg(short, long, default_value = "Player")]
    pub username: String,

    /// Player UUID
    #[arg(long, default_value = "00000000-0000-0000-0000-000000000000")]
    pub uuid: String,

    /// Target Minecraft server address (host:port)
    #[arg(short, long)]
    pub server: Option<String>,

    /// Path to Minecraft client assets directory
    #[arg(long)]
    pub assets_dir: Option<String>,

    /// Render distance in chunk radius
    #[arg(short, long, default_value_t = 8)]
    pub render_distance: u32,
}

struct GpuSectionMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

struct Player {
    pos: DVec3,
    velocity: DVec3,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
    bounding_box: Aabb,
}

impl Player {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let half_width = 0.3;
        let height = 1.8;
        let pos = DVec3::new(x, y, z);
        let bounding_box = Aabb::new(
            DVec3::new(x - half_width, y, z - half_width),
            DVec3::new(x + half_width, y + height, z + half_width),
        );
        Self {
            pos,
            velocity: DVec3::ZERO,
            yaw: -std::f32::consts::FRAC_PI_2,
            pitch: 0.0,
            on_ground: false,
            bounding_box,
        }
    }

    pub fn update_physics(&mut self, world: &World, dt: f64) {
        // Gravity
        self.velocity.y -= 32.0 * dt;
        if self.velocity.y < -78.4 {
            self.velocity.y = -78.4;
        }

        // Horizontal drag
        self.velocity.x *= 0.91;
        self.velocity.z *= 0.91;

        let delta = self.velocity * dt;

        // Collision detection against world voxels
        let min_block_x = (self.bounding_box.min.x + delta.x.min(0.0)).floor() as i32;
        let max_block_x = (self.bounding_box.max.x + delta.x.max(0.0)).ceil() as i32;
        let min_block_y = (self.bounding_box.min.y + delta.y.min(0.0)).floor() as i32;
        let max_block_y = (self.bounding_box.max.y + delta.y.max(0.0)).ceil() as i32;
        let min_block_z = (self.bounding_box.min.z + delta.z.min(0.0)).floor() as i32;
        let max_block_z = (self.bounding_box.max.z + delta.z.max(0.0)).ceil() as i32;

        let mut block_boxes = Vec::new();
        for bx in min_block_x..max_block_x {
            for by in min_block_y..max_block_y {
                for bz in min_block_z..max_block_z {
                    if world.get_block(bx, by, bz) != 0 {
                        block_boxes.push(Aabb::from_block(bx, by, bz));
                    }
                }
            }
        }

        // Y-axis movement & collision
        let mut dy = delta.y;
        for b in &block_boxes {
            dy = self.bounding_box.calculate_y_offset(b, dy);
        }
        self.bounding_box = self.bounding_box.offset(DVec3::new(0.0, dy, 0.0));
        self.on_ground = delta.y < 0.0 && dy != delta.y;
        if dy != delta.y {
            self.velocity.y = 0.0;
        }

        // X-axis movement & collision
        let mut dx = delta.x;
        for b in &block_boxes {
            dx = self.bounding_box.calculate_x_offset(b, dx);
        }
        self.bounding_box = self.bounding_box.offset(DVec3::new(dx, 0.0, 0.0));
        if dx != delta.x {
            self.velocity.x = 0.0;
        }

        // Z-axis movement & collision
        let mut dz = delta.z;
        for b in &block_boxes {
            dz = self.bounding_box.calculate_z_offset(b, dz);
        }
        self.bounding_box = self.bounding_box.offset(DVec3::new(0.0, 0.0, dz));
        if dz != delta.z {
            self.velocity.z = 0.0;
        }

        self.pos = DVec3::new(
            (self.bounding_box.min.x + self.bounding_box.max.x) * 0.5,
            self.bounding_box.min.y,
            (self.bounding_box.min.z + self.bounding_box.max.z) * 0.5,
        );
    }
}

struct AppState<'a> {
    window: Option<Arc<Window>>,
    render_ctx: Option<RenderContext<'a>>,
    camera: Camera,
    player: Player,
    world: Arc<World>,
    gpu_meshes: HashMap<(i32, i32, usize), GpuSectionMesh>,
    keys_down: HashMap<KeyCode, bool>,
    last_frame: Instant,
    args: CliArgs,
}

impl<'a> AppState<'a> {
    pub fn new(args: CliArgs) -> Self {
        let world = Arc::new(World::new());

        // Generate flat test terrain
        let radius = args.render_distance as i32;
        for cx in -radius..=radius {
            for cz in -radius..=radius {
                let mut chunk = ChunkColumn::new(cx, cz);
                for bx in 0..16 {
                    for bz in 0..16 {
                        // Bedrock layer
                        chunk.set_block(cx * 16 + bx, -64, cz * 16 + bz, 3);
                        // Stone layers
                        for by in -63..60 {
                            chunk.set_block(cx * 16 + bx, by, cz * 16 + bz, 3);
                        }
                        // Dirt layers
                        for by in 60..63 {
                            chunk.set_block(cx * 16 + bx, by, cz * 16 + bz, 2);
                        }
                        // Grass top layer
                        chunk.set_block(cx * 16 + bx, 63, cz * 16 + bz, 1);
                    }
                }
                world.insert_chunk(cx, cz, chunk);
            }
        }

        let player = Player::new(0.0, 65.0, 0.0);
        let camera = Camera::new(Vec3::new(0.0, 66.6, 0.0), 1200.0 / 720.0);

        Self {
            window: None,
            render_ctx: None,
            camera,
            player,
            world,
            gpu_meshes: HashMap::new(),
            keys_down: HashMap::new(),
            last_frame: Instant::now(),
            args,
        }
    }

    fn rebuild_visible_meshes(&mut self) {
        let Some(ctx) = &self.render_ctx else { return };
        let radius = self.args.render_distance as i32;

        for cx in -radius..=radius {
            for cz in -radius..=radius {
                if let Some(col_arc) = self.world.get_chunk(cx, cz) {
                    let col = col_arc.read();
                    for s_idx in 0..col.sections.len() {
                        let key = (cx, cz, s_idx);
                        if self.gpu_meshes.contains_key(&key) {
                            continue;
                        }

                        let sec = &col.sections[s_idx];
                        if sec.block_count == 0 {
                            continue;
                        }

                        let origin = glam::IVec3::new(
                            cx * 16,
                            MIN_Y + (s_idx as i32 * 16),
                            cz * 16,
                        );

                        let mesh = mesh_section(sec, origin, &self.world);
                        if mesh.is_empty() {
                            continue;
                        }

                        let vertex_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Section Vertex Buffer"),
                            contents: bytemuck::cast_slice(&mesh.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                        let index_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Section Index Buffer"),
                            contents: bytemuck::cast_slice(&mesh.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                        self.gpu_meshes.insert(
                            key,
                            GpuSectionMesh {
                                vertex_buffer,
                                index_buffer,
                                index_count: mesh.indices.len() as u32,
                            },
                        );
                    }
                }
            }
        }
    }
}

impl<'a> ApplicationHandler for AppState<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let win_attrs = WindowAttributes::default()
                .with_title(format!("HyperX Core — Minecraft 1.21.x (Rust Native) | {}", self.args.username))
                .with_inner_size(LogicalSize::new(1200, 720));

            let window = Arc::new(event_loop.create_window(win_attrs).expect("Failed to create window"));
            self.window = Some(window.clone());

            let ctx = pollster::block_on(RenderContext::new(window));
            self.render_ctx = Some(ctx);

            self.rebuild_visible_meshes();
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let sensitivity = 0.0025;
            self.player.yaw += (delta.0 as f32) * sensitivity;
            self.player.pitch -= (delta.1 as f32) * sensitivity;

            let max_pitch = 89.0f32.to_radians();
            self.player.pitch = self.player.pitch.clamp(-max_pitch, max_pitch);

            self.camera.yaw = self.player.yaw;
            self.camera.pitch = self.player.pitch;
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(ctx) = &mut self.render_ctx {
                    ctx.resize(new_size.width, new_size.height);
                    self.camera.aspect_ratio = new_size.width as f32 / new_size.height.max(1) as f32;
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                let is_down = state == ElementState::Pressed;
                self.keys_down.insert(key_code, is_down);

                if key_code == KeyCode::Escape && is_down {
                    event_loop.exit();
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - self.last_frame).as_secs_f64().min(0.05);
                self.last_frame = now;

                // Handle WASD player movement input
                let forward = self.camera.forward();
                let flat_forward = DVec3::new(forward.x as f64, 0.0, forward.z as f64).normalize_or_zero();
                let right = self.camera.right();
                let flat_right = DVec3::new(right.x as f64, 0.0, right.z as f64).normalize_or_zero();

                let mut move_dir = DVec3::ZERO;
                if *self.keys_down.get(&KeyCode::KeyW).unwrap_or(&false) {
                    move_dir += flat_forward;
                }
                if *self.keys_down.get(&KeyCode::KeyS).unwrap_or(&false) {
                    move_dir -= flat_forward;
                }
                if *self.keys_down.get(&KeyCode::KeyD).unwrap_or(&false) {
                    move_dir += flat_right;
                }
                if *self.keys_down.get(&KeyCode::KeyA).unwrap_or(&false) {
                    move_dir -= flat_right;
                }

                let speed = if *self.keys_down.get(&KeyCode::ControlLeft).unwrap_or(&false) {
                    12.0 // Sprint
                } else {
                    6.0 // Walk
                };

                if move_dir.length_squared() > 0.0 {
                    let move_norm = move_dir.normalize();
                    self.player.velocity.x += move_norm.x * speed * dt * 8.0;
                    self.player.velocity.z += move_norm.z * speed * dt * 8.0;
                }

                // Jump
                if *self.keys_down.get(&KeyCode::Space).unwrap_or(&false) && self.player.on_ground {
                    self.player.velocity.y = 9.0;
                    self.player.on_ground = false;
                }

                self.player.update_physics(&self.world, dt);

                // Update Camera to Player eye position
                self.camera.position = Vec3::new(
                    self.player.pos.x as f32,
                    (self.player.pos.y + 1.62) as f32,
                    self.player.pos.z as f32,
                );

                // Render frame
                if let Some(ctx) = &self.render_ctx {
                    ctx.update_uniforms(&self.camera);

                    let output = match ctx.surface.get_current_texture() {
                        Ok(texture) => texture,
                        Err(wgpu::SurfaceError::Lost) => {
                            let (_w, _h) = ctx.size;
                            // resize
                            return;
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            event_loop.exit();
                            return;
                        }
                        Err(_) => return,
                    };

                    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Terrain Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.65,
                                        g: 0.75,
                                        b: 0.90,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                                view: &ctx.depth_texture_view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: wgpu::StoreOp::Store,
                                }),
                                stencil_ops: None,
                            }),
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        render_pass.set_pipeline(&ctx.render_pipeline);
                        render_pass.set_bind_group(0, &ctx.uniform_bind_group, &[]);

                        for ((cx, cz, s_idx), mesh) in &self.gpu_meshes {
                            let origin = Vec3::new(
                                (*cx * 16) as f32,
                                (MIN_Y + (*s_idx as i32 * 16)) as f32,
                                (*cz * 16) as f32,
                            );
                            let end = origin + Vec3::splat(16.0);

                            // Frustum Culling
                            if !self.camera.is_box_visible(origin, end) {
                                continue;
                            }

                            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
                        }
                    }

                    ctx.queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                }

                if let Some(win) = &self.window {
                    win.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = CliArgs::parse();
    log::info!("Starting HyperX Native Rust Core for user '{}'", args.username);

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = AppState::new(args);
    let _ = event_loop.run_app(&mut app);
}
