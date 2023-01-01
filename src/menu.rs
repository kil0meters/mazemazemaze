use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    filter::FilterCamera,
    maze2d::{Maze2D, Maze2DBundle},
    state::AppState,
};

pub struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup_menu))
            .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(menu))
            .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(cleanup_menu));
    }
}

#[derive(Resource)]
struct MenuData {
    root: Entity,
}

#[derive(Component)]
struct MenuCamera;

#[derive(Component)]
struct CursorLight;

#[derive(Component)]
enum TitleButton {
    Maze3D,
    Maze2D,
}

fn setup_menu(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let root = commands
        .spawn(TransformBundle::default())
        .insert(VisibilityBundle::default())
        .with_children(|parent| {
            parent
                .spawn(Camera3dBundle {
                    camera: Camera {
                        priority: 0,
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, -Vec3::X),
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: PI / 8.0,
                        ..default()
                    }),
                    ..default()
                })
                .insert(MenuCamera)
                .insert(FilterCamera);

            let new_3d_maze = server.load("new_3d_maze.glb#Scene0");
            let new_2d_maze = server.load("new_2d_maze.glb#Scene0");

            parent
                .spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 3200.0,
                        shadows_enabled: true,
                        radius: 200.0,
                        color: Color::hex("ffff00").unwrap(),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 10.0, 0.0),
                    ..default()
                })
                .insert(CursorLight);

            parent
                .spawn(SceneBundle {
                    scene: new_3d_maze,
                    transform: Transform {
                        translation: Vec3::new(-1.5, 0.0, 0.0),
                        rotation: Quat::from_rotation_z(PI / 2.0),
                        scale: Vec3::new(0.2, 0.2, 0.2),
                    },
                    ..default()
                })
                .insert(Collider::cuboid(4.0, 5.0, 35.0))
                .insert(TitleButton::Maze3D);

            parent
                .spawn(SceneBundle {
                    scene: new_2d_maze,
                    transform: Transform {
                        translation: Vec3::new(1.5, 0.0, 0.0),
                        rotation: Quat::from_rotation_z(PI / 2.0),
                        scale: Vec3::new(0.2, 0.2, 0.2),
                    },
                    ..default()
                })
                .insert(Collider::cuboid(4.0, 5.0, 35.0))
                .insert(TitleButton::Maze2D);

            parent.spawn(Maze2DBundle {
                maze: Maze2D::hunt_and_kill(18, 32),

                // We need to preset a mesh here don't ask
                mesh: meshes.add(shape::Cube::new(50.0).into()),

                material: materials.add(Color::hex("ffff00").unwrap().into()),

                transform: Transform {
                    translation: Vec3::new(-10.0, 0.0, -20.0),
                    scale: Vec3::new(0.2, 0.2, 0.2),
                    ..default()
                },
                ..default()
            });
        })
        .id();

    commands.insert_resource(MenuData { root });
}

fn menu(
    mut windows: ResMut<Windows>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MenuCamera>>,
    rapier_context: Res<RapierContext>,
    mut state: ResMut<State<AppState>>,
    buttons: Res<Input<MouseButton>>,
    mut light_query: Query<&mut Transform, (With<CursorLight>, Without<TitleButton>)>,
    mut title_text_query: Query<(Entity, &mut Transform, &TitleButton), Without<CursorLight>>,
) {
    let window = windows.get_primary_mut().unwrap();

    window.set_cursor_icon(CursorIcon::Arrow);

    if let Some(mouse_position) = window.cursor_position() {
        #[cfg(not(target_family = "wasm"))]
        let mouse_position = mouse_position * window.scale_factor() as f32;

        for (camera, camera_transform) in camera_query.iter() {
            if let Some(ray) = camera.viewport_to_world(camera_transform, mouse_position) {
                // intersect ray with plane at y = 10
                let t = (Vec3::new(0.0, 10.0, 0.0) - ray.origin).dot(Vec3::Y)
                    / ray.direction.dot(Vec3::Y);

                let cursor_position_world = ray.origin + (ray.direction * t);

                // update light position
                for mut light_transform in light_query.iter_mut() {
                    light_transform.translation = cursor_position_world;
                }

                let collision = rapier_context.cast_ray(
                    ray.origin,
                    ray.direction,
                    100.0,
                    true,
                    QueryFilter::new(),
                );

                for (entity, mut translation, button) in title_text_query.iter_mut() {
                    // update scale of title text based on distance from cursor
                    let scale = (20.0
                        / translation
                            .translation
                            .distance(cursor_position_world)
                            .powi(2))
                    .clamp(0.05, 0.2);

                    translation.scale = Vec3::from([scale; 3]);

                    if matches!(collision, Some((e, _)) if e == entity) {
                        window.set_cursor_icon(CursorIcon::Hand);

                        if buttons.just_released(MouseButton::Left) {
                            match button {
                                TitleButton::Maze3D => {
                                    bevy::log::info!("3D Maze Mode");
                                    state.set(AppState::Maze3D).unwrap();
                                }
                                TitleButton::Maze2D => {
                                    bevy::log::info!("2D Maze Mode");
                                    state.set(AppState::Maze2D).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.root).despawn_recursive();
}
