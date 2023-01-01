use bevy::{
    asset::load_internal_asset,
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::RenderTarget,
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    window::WindowResized,
};

// heavily based upon
// https://github.com/annieversary/bevy_color_blindness/blob/main/src/lib.rs

const FILTER_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 10868367484578534037);

pub struct FilterPlugin;
impl Plugin for FilterPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(app, FILTER_SHADER_HANDLE, "filter.wgsl", Shader::from_wgsl);

        app.add_plugin(Material2dPlugin::<FilterMaterial>::default())
            .add_system(setup_filter_camera)
            .add_system(update_quad)
            .add_system(update_block_size);
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "8ce8b2f8-8bf5-4688-b114-666afb2f2fc8"]
struct FilterMaterial {
    // blame wasm for this being a vec4
    #[uniform(0)]
    block_size: Vec4,

    #[texture(1)]
    #[sampler(2)]
    source_image: Handle<Image>,
}

impl Material2d for FilterMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(FILTER_SHADER_HANDLE.typed())
    }
}

#[derive(Component)]
pub struct FilterCamera;

#[derive(Component)]
struct FilterQuad;

fn setup_filter_camera(
    mut commands: Commands,
    windows: Res<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FilterMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<(Entity, &mut Camera), Added<FilterCamera>>,
) {
    for (entity, mut camera) in cameras.iter_mut() {
        let size = match &camera.target {
            RenderTarget::Window(window_id) => {
                let window = windows.get(*window_id).unwrap();
                Extent3d {
                    width: window.physical_width(),
                    height: window.physical_height(),
                    ..default()
                }
            }
            RenderTarget::Image(handle) => {
                let image = images.get(handle).unwrap();
                image.texture_descriptor.size
            }
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };

        image.resize(size);

        let image_handle = images.add(image);

        let post_processing_pass_layer =
            RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            size.width as f32,
            size.height as f32,
        ))));

        let material_handle = materials.add(FilterMaterial {
            source_image: image_handle.clone(),
            block_size: Vec4::new(8.0, 0.0, 0.0, 0.0),
        });

        let original_target = camera.target.clone();
        camera.target = RenderTarget::Image(image_handle);

        commands
            .entity(entity)
            .insert(material_handle.clone())
            .insert(UiCameraConfig { show_ui: false })
            .insert(VisibilityBundle::default())
            .with_children(|parent| {
                parent
                    .spawn(FilterQuad)
                    .insert(MaterialMesh2dBundle {
                        mesh: quad_handle.into(),
                        material: material_handle,
                        transform: Transform {
                            translation: Vec3::new(0.0, 0.0, 1.5),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .insert(post_processing_pass_layer);

                parent
                    .spawn(Camera2dBundle {
                        camera: Camera {
                            // renders after the first main camera which has default value: 0.
                            priority: 1,
                            // set this new camera to render to where the other camera was rendering
                            target: original_target,
                            ..Default::default()
                        },
                        ..Camera2dBundle::default()
                    })
                    .insert(post_processing_pass_layer);
            });
    }
}

fn update_block_size(mut materials: ResMut<Assets<FilterMaterial>>, windows: Res<Windows>) {
    let new_block_size = 3.0 * windows.get_primary().unwrap().scale_factor() as f32;

    for material in materials.iter_mut() {
        if material.1.block_size.x != new_block_size {
            material.1.block_size.x = new_block_size;
        }
    }
}

fn update_quad(
    windows: Res<Windows>,
    mut resize_reader: EventReader<WindowResized>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<FilterMaterial>>,
    query: Query<&Mesh2dHandle, With<FilterQuad>>,
) {
    let window = windows.get_primary().unwrap();

    for _ in resize_reader.iter() {
        for quad in query.iter() {
            let new_mesh = Mesh::from(shape::Quad::new(Vec2::new(
                window.physical_width() as f32,
                window.physical_height() as f32,
            )));

            *meshes.get_mut(&quad.0).unwrap() = new_mesh;
        }

        for material in materials.iter_mut() {
            images
                .get_mut(&material.1.source_image)
                .unwrap()
                .resize(Extent3d {
                    width: window.physical_width(),
                    height: window.physical_height(),
                    ..Default::default()
                });
        }
    }
}
