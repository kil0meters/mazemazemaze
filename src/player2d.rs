use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_rapier3d::prelude::*;

use crate::{filter::FilterCamera, state::AppState, Goal, Settings};

#[derive(Component)]
pub struct Player2D {
    velocity: Vec3,
    grounded: bool,

    speed: f32,
    gravity: f32,
    jump_power: f32,
}

impl Default for Player2D {
    fn default() -> Self {
        Player2D {
            velocity: Vec3::ZERO,
            grounded: false,

            speed: 12.0,
            gravity: 30.0,
            jump_power: 10.8,
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
        .spawn(Player2D::default())
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::capsule_y(0.5, 1.0))
        .insert(VisibilityBundle::default())
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 2.0, 0.0)))
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC)
        .with_children(|parent| {
            parent
                .spawn(Camera3dBundle {
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: PI / 2.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
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
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            });
        });
}

fn cleanup_player(
    mut windows: ResMut<Windows>,
    mut commands: Commands,
    query: Query<Entity, With<Player2D>>,
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
    mut query: Query<(&Transform, &mut Player2D)>,
) {
    if let Some(window) = windows.get_primary() {
        for (transform, mut player) in query.iter_mut() {
            let mut velocity = Vec3::ZERO;

            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            for key in keys.get_pressed() {
                match window.cursor_grab_mode() {
                    CursorGrabMode::None => (),
                    _ => match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::S => velocity -= forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::D => velocity += right,
                        KeyCode::Space => {
                            if player.grounded {
                                player.velocity.y += player.jump_power;
                                player.grounded = false;
                            }
                        }
                        _ => (),
                    },
                }
            }

            velocity = velocity.normalize_or_zero() * player.speed;

            player.velocity.x = velocity.x;
            player.velocity.z = velocity.z;
        }
    }
}

fn player_gravity(time: Res<Time>, mut query: Query<&mut Player2D>) {
    for mut player in query.iter_mut() {
        if !player.grounded {
            player.velocity.y -= time.delta_seconds() * player.gravity;
        } else {
            player.velocity.y = 0.0;
        }
    }
}

fn player_velocity(
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
    mut state: ResMut<State<AppState>>,
    mut query: Query<(Entity, &Collider, &mut Transform, &mut Player2D)>,
    goal_query: Query<Entity, With<Goal>>,
) {
    let max_toi = 4.0;

    for (entity, collider, mut transform, mut player) in query.iter_mut() {
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

                if (1.0 - normal.dot(Vec3::Y)).abs() < 0.01 {
                    player.grounded = true;
                }

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
    mut motion: EventReader<MouseMotion>,
    mut player_query: Query<&mut Transform, (With<Player2D>, Without<PlayerCamera>)>,
    mut camera_query: Query<&mut Transform, (Without<Player2D>, With<PlayerCamera>)>,
) {
    let window = windows.get_primary().unwrap();

    let mut new_yaw = 0.0;
    let mut new_pitch = 0.0;

    for ev in motion.iter() {
        match window.cursor_grab_mode() {
            CursorGrabMode::None => (),
            _ => {
                new_pitch -= (settings.sensitivity * ev.delta.y).to_radians();
                new_yaw -= (settings.sensitivity * ev.delta.x).to_radians();
            }
        }
    }

    new_pitch = new_pitch.clamp(-1.54, 1.54);

    for mut player_transform in player_query.iter_mut() {
        player_transform.rotation *= Quat::from_axis_angle(Vec3::Y, new_yaw)
    }

    for mut camera_transform in camera_query.iter_mut() {
        camera_transform.rotation *= Quat::from_axis_angle(Vec3::X, new_pitch);
    }
}

pub struct Player2DPlugin;

impl Plugin for Player2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::Maze2D).with_system(setup_player))
            .add_system_set(SystemSet::on_exit(AppState::Maze2D).with_system(cleanup_player))
            .add_system_set(
                SystemSet::on_update(AppState::Maze2D)
                    .with_system(player_look)
                    .with_system(player_velocity)
                    .with_system(player_gravity.after(player_velocity))
                    .with_system(player_move.after(player_velocity)),
            );
    }
}
