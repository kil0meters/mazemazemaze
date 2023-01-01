#import bevy_pbr::mesh_view_bindings

struct FilterMaterial {
    block_size: vec4<f32>,
};


@group(1) @binding(0)
var<uniform> material: FilterMaterial;

@group(1) @binding(1)
var texture: texture_2d<f32>;

@group(1) @binding(2)
var source_image: sampler;


fn color_dist(a: vec3<f32>, b: vec3<f32>) -> f32 {
    return 0.3 * (a.r - b.r) * (a.r - b.r) + //
           0.59 * (a.g - b.g) * (a.g - b.g) + //
           0.11 * (a.b - b.b) * (a.b - b.b);
}

fn index_value(position: vec2<f32>) -> f32 {
    var index_matrix: array<f32, 64> = array<f32, 64>(
        0.0,
        32.0,
        8.0,
        40.0,
        2.0,
        34.0,
        10.0,
        42.0,
        48.0,
        16.0,
        56.0,
        24.0,
        50.0,
        18.0,
        58.0,
        26.0,
        12.0,
        44.0,
        4.0,
        36.0,
        14.0,
        46.0,
        6.0,
        38.0,
        60.0,
        28.0,
        52.0,
        20.0,
        62.0,
        30.0,
        54.0,
        22.0,
        3.0,
        35.0,
        11.0,
        43.0,
        1.0,
        33.0,
        9.0,
        41.0,
        51.0,
        19.0,
        59.0,
        27.0,
        49.0,
        17.0,
        57.0,
        25.0,
        15.0,
        47.0,
        7.0,
        39.0,
        13.0,
        45.0,
        5.0,
        37.0,
        63.0,
        31.0,
        55.0,
        23.0,
        61.0,
        29.0,
        53.0,
        21.0
    );

    return index_matrix[u32(position.x) % 8u * 8u + u32(position.y) % 8u];
}

fn dither(
    position: vec2<f32>,
    pixel: vec3<f32>
) -> vec3<f32> {
    // saga master system color palette
    // https://en.wikipedia.org/wiki/List_of_video_game_console_palettes
    var palette: array<vec3<f32>, 64> = array<vec3<f32>, 64>(
        vec3<f32>(0.00, 0.00, 0.00),
        vec3<f32>(0.00, 0.00, 0.33),
        vec3<f32>(0.00, 0.00, 0.67),
        vec3<f32>(0.00, 0.00, 1.00),
        vec3<f32>(0.33, 0.00, 0.00),
        vec3<f32>(0.33, 0.00, 0.33),
        vec3<f32>(0.33, 0.00, 0.67),
        vec3<f32>(0.33, 0.00, 1.00),
        vec3<f32>(0.67, 0.00, 0.00),
        vec3<f32>(0.67, 0.00, 0.33),
        vec3<f32>(0.67, 0.00, 0.67),
        vec3<f32>(0.67, 0.00, 1.00),
        vec3<f32>(1.00, 0.00, 0.00),
        vec3<f32>(1.00, 0.00, 0.33),
        vec3<f32>(1.00, 0.00, 0.67),
        vec3<f32>(1.00, 0.00, 1.00),
        vec3<f32>(0.00, 0.33, 0.00),
        vec3<f32>(0.00, 0.33, 0.33),
        vec3<f32>(0.00, 0.33, 0.67),
        vec3<f32>(0.00, 0.33, 1.00),
        vec3<f32>(0.33, 0.33, 0.00),
        vec3<f32>(0.33, 0.33, 0.33),
        vec3<f32>(0.33, 0.33, 0.67),
        vec3<f32>(0.33, 0.33, 1.00),
        vec3<f32>(0.67, 0.33, 0.00),
        vec3<f32>(0.67, 0.33, 0.33),
        vec3<f32>(0.67, 0.33, 0.67),
        vec3<f32>(0.67, 0.33, 1.00),
        vec3<f32>(1.00, 0.33, 0.00),
        vec3<f32>(1.00, 0.33, 0.33),
        vec3<f32>(1.00, 0.33, 0.67),
        vec3<f32>(1.00, 0.33, 1.00),
        vec3<f32>(0.00, 0.67, 0.00),
        vec3<f32>(0.00, 0.67, 0.33),
        vec3<f32>(0.00, 0.67, 0.67),
        vec3<f32>(0.00, 0.67, 1.00),
        vec3<f32>(0.33, 0.67, 0.00),
        vec3<f32>(0.33, 0.67, 0.33),
        vec3<f32>(0.33, 0.67, 0.67),
        vec3<f32>(0.33, 0.67, 1.00),
        vec3<f32>(0.67, 0.67, 0.00),
        vec3<f32>(0.67, 0.67, 0.33),
        vec3<f32>(0.67, 0.67, 0.67),
        vec3<f32>(0.67, 0.67, 1.00),
        vec3<f32>(1.00, 0.67, 0.00),
        vec3<f32>(1.00, 0.67, 0.33),
        vec3<f32>(1.00, 0.67, 0.67),
        vec3<f32>(1.00, 0.67, 1.00),
        vec3<f32>(0.00, 1.00, 0.00),
        vec3<f32>(0.00, 1.00, 0.33),
        vec3<f32>(0.00, 1.00, 0.67),
        vec3<f32>(0.00, 1.00, 1.00),
        vec3<f32>(0.33, 1.00, 0.00),
        vec3<f32>(0.33, 1.00, 0.33),
        vec3<f32>(0.33, 1.00, 0.67),
        vec3<f32>(0.33, 1.00, 1.00),
        vec3<f32>(0.67, 1.00, 0.00),
        vec3<f32>(0.67, 1.00, 0.33),
        vec3<f32>(0.67, 1.00, 0.67),
        vec3<f32>(0.67, 1.00, 1.00),
        vec3<f32>(1.00, 1.00, 0.00),
        vec3<f32>(1.00, 1.00, 0.33),
        vec3<f32>(1.00, 1.00, 0.67),
        vec3<f32>(1.00, 1.00, 1.00),
    );

    var closest_color: vec3<f32> = palette[0];
    var closest_color_dist: f32 = 2000000.0;

    var second_closest_color: vec3<f32> = closest_color;
    var second_closest_color_dist: f32 = 1000000.0;

    for (var i: u32 = 0u; i < 64u; i++) {
        var tmp_dist = color_dist(pixel, palette[i]);
        if tmp_dist < closest_color_dist {
            second_closest_color = closest_color;
            second_closest_color_dist = closest_color_dist;

            closest_color = palette[i];
            closest_color_dist = tmp_dist;
        } else if tmp_dist < second_closest_color_dist {
            second_closest_color = palette[i];
            second_closest_color_dist = tmp_dist;
        }
    }

    // actual dithering
    let d = index_value(position);
    let diff = (30.0 * closest_color_dist) / second_closest_color_dist;

    if d < diff {
        return second_closest_color;
    } else {
        return closest_color;
    }
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    // Get screen position with coordinates from 0 to 1
    var uv = position.xy / view.viewport.zw;
    let adjusted_dimensions = view.viewport.zw / material.block_size.x;
    let adjusted_position = floor((uv + 0.5) * adjusted_dimensions);
    uv = adjusted_position / adjusted_dimensions - 0.5;

    var c = textureSample(texture, source_image, uv);

    return vec4<f32>(
        dither(adjusted_position, c.rgb),
        1.0
    );
}
