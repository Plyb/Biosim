#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var material_color_texture: texture_2d<f32>;
@group(2) @binding(1) var material_color_sampler: sampler;

const sqrt_3 : f32 = 1.732050807568877;
const WORLD_WIDTH : f32 = 32;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let u = mesh.uv.x * 3.0;
    let v = 1.0 - mesh.uv.y;

    var column = u32(floor(u * WORLD_WIDTH));
    let in_even_column = column % 2 == 0;
    let offset = select(0.5, 0.0, in_even_column);
    var row = u32(floor(0.5 * v * WORLD_WIDTH - offset));

    let x_in_square = u * WORLD_WIDTH - f32(column);
    let y_in_square = 0.5 * v * WORLD_WIDTH - offset - f32(row);
    let possibly_out_of_hex = x_in_square > 0.66667;
    if (possibly_out_of_hex) {
        let parameter_upper = y_in_square + 1.5 * x_in_square - 2.0;
        let parameter_lower = y_in_square - 1.5 * x_in_square + 1.0;
        if (parameter_upper > 0 || parameter_lower < 0) {
            column++;
        }
        if (parameter_upper > 0 && !in_even_column) {
            row++;
        }
        if (parameter_lower < 0 && in_even_column) {
            row--;
        }
    }

    let hexel_x: u32 = (column / 2) - row;
    let hexel_y: u32 = row * 2 + u32(select(1, 0, column % 2 == 0));
    

    // let hexel_u = 1.5 * u - 0.5 * v;
    // let hexel_v = v;

    // let x = 32 * (.75*u - 0.25*v);
    // let y = 32 * (.25*u + 0.75*v);
    // let z = x-y;

    // // let axial_x = u32(round((floor(x-y)-floor(z-x))/3.0));
    // // let axial_y = u32(round((floor(y-z)-floor(x-y))/3.0));
    // // let axial_z = u32(round((floor(z-x)-floor(y-z))/3.0));

    // let ix = round(z);
    // let iy = round(y);

    if (hexel_x < 0 || hexel_x > u32(WORLD_WIDTH) || hexel_y < 0 || hexel_y > u32(WORLD_WIDTH)) {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    // return vec4<f32>(select(0.0, 1.0, hexel_x == 2), select(0.0, 1.0, hexel_y == 2) / WORLD_WIDTH, 0.0, 1.0);
    // // return vec4<f32>(u, v, 0.0, 1.0);

    return textureSample(material_color_texture, material_color_sampler, vec2<f32>((f32(hexel_x) + 0.5) / WORLD_WIDTH, (f32(hexel_y) + 0.5) / WORLD_WIDTH));
}
