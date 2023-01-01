use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_rapier3d::prelude::*;

use crate::{filter::FilterCamera, state::AppState, Goal, Settings};

#[derive(Component)]
pub struct Player3D {
    velocity: Vec3,
    speed: f32,
}

impl Default for Player3D {
    fn default() -> Self {
        Player3D {
            velocity: Vec3::ZERO,
            speed: 12.0,
        }
    }
}

#[derive(Component)]
struct PlayerCamera;

fn setup_player(mut commands: Commands, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();

    // grab cursor
    window.set_cursor_grab_mode(CursorGrabMode::Locked);
    window.set_cursor_visibility(false);

    commands
        .spawn(Player3D::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(1.0))
        .insert(VisibilityBundle::default())
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 2.0, 0.0)))
        .with_children(|parent| {
            parent
                .spawn(Camera3dBundle {
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: PI / 2.0,
                        ..default()
                    }),
                    ..default()
                })
                .insert(PlayerCamera)
                .insert(FilterCamera);

            parent.spawn(PointLightBundle {
                point_light: PointLight {
                    intensity: 800.0,
                    shadows_enabled: true,
                    radius: 0.1,
                    ..default()
                },
                ..default()
            });
        });
}

fn cleanup_player(
    mut windows: ResMut<Windows>,
    mut commands: Commands,
    query: Query<Entity, With<Player3D>>,
) {
    let window = windows.get_primary_mut().unwrap();

    // ungrab cursor
    window.set_cursor_grab_mode(CursorGrabMode::None);
    window.set_cursor_visibility(true);

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn player_move(
    keys: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut query: Query<(&Transform, &mut Player3D)>,
) {
    if let Some(window) = windows.get_primary() {
        for (transform, mut player) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;

            let forward = transform.forward();
            let right = transform.right();
            let up = transform.up();

            for key in keys.get_pressed() {
                match window.cursor_grab_mode() {
                    CursorGrabMode::None => (),
                    _ => match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::S => velocity -= forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::D => velocity += right,
                        KeyCode::Space => velocity += up,
                        KeyCode::LShift => velocity -= up,
                        _ => (),
                    },
                }
            }

            velocity = velocity.normalize_or_zero() * player.speed;

            player.velocity.x = velocity.x;
            player.velocity.y = velocity.y;
            player.velocity.z = velocity.z;
        }
    }
}

fn player_velocity(
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<(Entity, &Collider, &mut Transform, &Player3D)>,
    goal_query: Query<Entity, With<Goal>>,
) {
    let max_toi = 4.0;

    for (entity, collider, mut transform, player) in query.iter_mut() {
        let mut velocity = Some(time.delta_seconds() * player.velocity);
        let filter = QueryFilter::default().exclude_collider(entity);

        while let Some(adjusted_velocity) = velocity {
            if let Some((entity, collision)) = rapier_context.cast_shape(
                transform.translation,
                transform.rotation,
                adjusted_velocity,
                collider,
                max_toi,
                filter,
            ) {
                // test if entity is a goal component
                if goal_query.get(entity).is_ok() {
                    state.set(AppState::MainMenu).unwrap();
                    println!("You won!");
                }

                let normal = collision.normal1;

                // slide along wall of collision
                velocity = Some(adjusted_velocity - (normal * adjusted_velocity.dot(normal)));
            } else {
                transform.translation += adjusted_velocity;
                velocity = None;
            }
        }
    }
}

fn player_look(
    windows: Res<Windows>,
    settings: Res<Settings>,
    buttons: Res<Input<MouseButton>>,
    time: Res<Time>,
    mut motion: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player3D>>,
) {
    let window = windows.get_primary().unwrap();

    let mut new_yaw = 0.0;
    let mut new_pitch = 0.0;
    let mut new_roll = 0.0;

    for ev in motion.iter() {
        match window.cursor_grab_mode() {
            CursorGrabMode::None => (),
            _ => {
                new_pitch -= (settings.sensitivity * ev.delta.y).to_radians();
                new_yaw -= (settings.sensitivity * ev.delta.x).to_radians();
            }
        }
    }

    if buttons.pressed(MouseButton::Left) {
        new_roll += 2.0 * time.delta_seconds();
    }

    if buttons.pressed(MouseButton::Right) {
        new_roll -= 2.0 * time.delta_seconds();
    }

    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_euler(EulerRot::YXZ, new_yaw, new_pitch, new_roll);
    }
}

pub struct Player3DPlugin;

impl Plugin for Player3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Maze3D).with_system(setup_player))
            .add_system_set(SystemSet::on_exit(AppState::Maze3D).with_system(cleanup_player))
            .add_system_set(
                SystemSet::on_update(AppState::Maze3D)
                    .with_system(player_look)
                    .with_system(player_move)
                    .with_system(player_velocity),
            );
    }
}
