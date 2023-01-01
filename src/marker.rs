use bevy::prelude::*;

use crate::{player2d::Player2D, player3d::Player3D, SpinBouncing};

#[derive(Component)]
pub struct Marker;

fn spawn_marker(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    keys: Res<Input<KeyCode>>,
    query: Query<&Transform, Or<(With<Player3D>, With<Player2D>)>>,
) {
    for transform in query.iter() {
        if keys.just_pressed(KeyCode::E) {
            commands
                .spawn(PbrBundle {
                    mesh: meshes.add(
                        shape::UVSphere {
                            radius: 0.5,
                            ..default()
                        }
                        .into(),
                    ),
                    transform: Transform {
                        translation: transform.translation.clone(),
                        ..default()
                    },
                    material: materials.add(Color::hex("ffff00").unwrap().into()),
                    visibility: Visibility::INVISIBLE,
                    ..default()
                })
                .insert(SpinBouncing)
                .insert(Marker);
        }
    }
}

fn marker_visibility(
    mut marker_query: Query<(&GlobalTransform, &mut Visibility), With<Marker>>,
    player_query: Query<&GlobalTransform, Or<(With<Player3D>, With<Player2D>)>>,
) {
    for player_transform in player_query.iter() {
        for (transform, mut visibility) in marker_query.iter_mut() {
            if player_transform
                .translation()
                .distance(transform.translation())
                < 0.5
            {
                visibility.is_visible = false;
            } else {
                visibility.is_visible = true;
            }
        }
    }
}

pub struct MarkerPlugin;
impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(spawn_marker).add_system(marker_visibility);
    }
}
