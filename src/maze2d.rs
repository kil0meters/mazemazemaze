use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::render_resource::PrimitiveTopology;
use bevy_rapier3d::prelude::*;
use block_mesh::ndshape::{RuntimeShape, Shape};
use block_mesh::{
    greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};

use crate::direction::Direction2D;
use crate::marker::Marker;
use crate::state::AppState;
use crate::{Goal, SpinBouncing};

const MAZE_SCALE: f32 = 5.0;

#[derive(Component, Default)]
pub struct Maze2D {
    // true == filled in/not walkable
    cells: Vec<Vec<bool>>,

    // logical size
    width: usize,
    height: usize,
    // bevy shit
}

impl Maze2D {
    // generates a maze using hunt and kill
    pub fn hunt_and_kill(width: usize, height: usize) -> Maze2D {
        let mut maze = Maze2D {
            cells: vec![vec![true; width * 2 + 1]; height * 2 + 1],
            width,
            height,
        };

        maze.random_walk(1, 1);

        for y in (1..(maze.height * 2)).step_by(2) {
            for x in (1..(maze.width * 2)).step_by(2) {
                if maze.cells[y][x] {
                    for direction in Direction2D::random_order() {
                        match direction {
                            Direction2D::Up => {
                                if y > 1 && !maze.cells[y - 2][x] {
                                    maze.cells[y - 1][x] = false;
                                    break;
                                }
                            }

                            Direction2D::Down => {
                                if y < maze.height * 2 - 1 && !maze.cells[y + 2][x] {
                                    maze.cells[y + 1][x] = false;
                                    break;
                                }
                            }

                            Direction2D::Left => {
                                if x > 1 && !maze.cells[y][x - 2] {
                                    maze.cells[y][x - 1] = false;
                                    break;
                                }
                            }

                            Direction2D::Right => {
                                if x < maze.width * 2 - 1 && !maze.cells[y][x + 2] {
                                    maze.cells[y][x + 1] = false;
                                    break;
                                }
                            }
                        }
                    }

                    maze.random_walk(x, y);
                }
            }
        }

        maze
    }

    fn random_walk(&mut self, x: usize, y: usize) {
        let mut next = Some((x, y));

        while let Some((x, y)) = next.take() {
            self.cells[y][x] = false;

            for direction in Direction2D::random_order() {
                match direction {
                    Direction2D::Up => {
                        if y > 1 && self.cells[y - 2][x] {
                            self.cells[y - 1][x] = false;
                            next = Some((x, y - 2));
                        }
                    }

                    Direction2D::Down => {
                        if y < self.height * 2 - 1 && self.cells[y + 2][x] {
                            self.cells[y + 1][x] = false;
                            next = Some((x, y + 2));
                        }
                    }

                    Direction2D::Left => {
                        if x > 1 && self.cells[y][x - 2] {
                            self.cells[y][x - 1] = false;
                            next = Some((x - 2, y));
                        }
                    }

                    Direction2D::Right => {
                        if x < self.width * 2 - 1 && self.cells[y][x + 2] {
                            self.cells[y][x + 1] = false;
                            next = Some((x + 2, y));
                        }
                    }
                }

                if next.is_some() {
                    break;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
struct BoolVoxel(bool);

const EMPTY: BoolVoxel = BoolVoxel(false);
const FULL: BoolVoxel = BoolVoxel(true);

impl Voxel for BoolVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == EMPTY {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for BoolVoxel {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

#[derive(Bundle, Default)]
pub struct Maze2DBundle {
    pub maze: Maze2D,
    pub collider: Collider,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

fn generate_maze2d_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<
        (
            &Maze2D,
            &Handle<Mesh>,
            &Handle<StandardMaterial>,
            &mut Transform,
            &mut Collider,
        ),
        Added<Maze2D>,
    >,
) {
    for (maze, mesh_handle, material_handle, mut transform, mut collider) in query.iter_mut() {
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        let shape =
            RuntimeShape::<u32, 3>::new([maze.width as u32 * 2 + 3, 3, maze.height as u32 * 2 + 3]);

        // This chunk will cover just a single octant of a sphere SDF (radius 15).
        let mut voxels = vec![EMPTY; shape.size() as usize];
        let mut cubes = vec![];

        for y in 0..maze.cells.len() {
            for x in 0..maze.cells[0].len() {
                if maze.cells[y][x] {
                    voxels[shape.linearize([x as u32 + 1, 1, y as u32 + 1]) as usize] = FULL;

                    cubes.push((
                        Vec3::new(x as f32 + 1.5, 1.5, y as f32 + 1.5),
                        Rot::default(),
                        Collider::cuboid(0.5, 0.5, 0.5),
                    ));
                }
            }
        }

        let mut buffer = GreedyQuadsBuffer::new(voxels.len());
        greedy_quads(
            &voxels,
            &shape,
            [0, 0, 0],
            [maze.width as u32 * 2 + 2, 2, maze.height as u32 * 2 + 2],
            &faces,
            &mut buffer,
        );

        let num_indices = buffer.quads.num_quads() * 6;
        let num_vertices = buffer.quads.num_quads() * 4;
        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        for (group, face) in buffer.quads.groups.into_iter().zip(faces.into_iter()) {
            for quad in group.into_iter() {
                indices.extend_from_slice(&face.quad_mesh_indices(positions.len() as u32));
                positions.extend_from_slice(&face.quad_mesh_positions(&quad, 1.0));
                normals.extend_from_slice(&face.quad_mesh_normals());
            }
        }

        let mut render_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions.clone()),
        );
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        );
        render_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float32x2(vec![[0.0; 2]; num_vertices]),
        );
        render_mesh.set_indices(Some(Indices::U32(indices.clone())));

        meshes.set_untracked(mesh_handle, render_mesh);
        materials.set_untracked(
            material_handle,
            StandardMaterial {
                base_color: Color::hex("0000ff").unwrap(),
                perceptual_roughness: 0.9,
                metallic: 0.0,
                ..default()
            },
        );

        transform.translation += Vec3::new(-MAZE_SCALE * 2.5, -MAZE_SCALE, -MAZE_SCALE * 2.5);
        transform.scale *= Vec3::from([MAZE_SCALE; 3]);

        *collider = Collider::compound(cubes);
    }
}

fn setup_maze2d(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(Maze2DBundle {
            maze: Maze2D::hunt_and_kill(10, 10),

            // We need to preset a mesh here don't ask
            mesh: meshes.add(shape::Cube::new(50.0).into()),

            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(PbrBundle {
                    mesh: meshes.add(shape::Plane { size: 100.0 }.into()),
                    material: materials.add(StandardMaterial {
                        base_color: Color::hex("00aaff").unwrap(),
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 1.0, 0.0),
                    ..default()
                })
                .insert(Collider::cuboid(100.0, 0.01, 100.0));
        });

    let scene = server.load("goal.glb#Scene0");

    commands
        .spawn(SceneBundle {
            scene,
            transform: Transform {
                translation: Vec3::new(90.0, 2.0, 90.0),
                scale: Vec3::new(0.3, 0.3, 0.3),
                ..default()
            },
            ..default()
        })
        .insert(Collider::ball(1.0))
        .insert(SpinBouncing)
        .insert(Goal);
}

fn cleanup_maze2d(
    mut commands: Commands,
    query: Query<Entity, Or<(With<Goal>, With<Maze2D>, With<Marker>)>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct Maze2DPlugin;
impl Plugin for Maze2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(generate_maze2d_mesh)
            .add_system_set(SystemSet::on_enter(AppState::Maze2D).with_system(setup_maze2d))
            .add_system_set(SystemSet::on_exit(AppState::Maze2D).with_system(cleanup_maze2d));
    }
}
