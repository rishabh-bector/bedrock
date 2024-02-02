@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var vertices: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), // Bottom-left
        vec2<f32>(1.0, -1.0),  // Bottom-right
        vec2<f32>(-1.0, 1.0),  // Top-left
        vec2<f32>(-1.0, 1.0),  // Top-left
        vec2<f32>(1.0, -1.0),  // Bottom-right
        vec2<f32>(1.0, 1.0)    // Top-right
    );
    return vec4<f32>(vertices[in_vertex_index], 0.0, 1.0);
}

@group(0)
@binding(1)
var r_color: texture_2d<f32>;

@group(1)
@binding(0)
var<uniform> ffff: vec2<f32>;

struct FragmentOutput {
  @location(0) color0: vec4<f32>,
  @location(1) color1: vec4<f32>
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> FragmentOutput {
    var out: FragmentOutput;
    let uv: vec2<i32> = vec2<i32>(frag_coord.xy);

    let currentCell: vec4<f32> = textureLoad(r_color, uv, 0);

    let textureSizeU: vec2<u32> = textureDimensions(r_color, 0);
    let textureSize: vec2<i32> = vec2<i32>(i32(textureSizeU.x), i32(textureSizeU.y));

    // Calculate the neighboring cell coordinates
    let l1u1: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(-1, -1), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
    let l0u1: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(0, -1), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
    let r1u1: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(1, -1), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
    let l1u0: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(-1, 0), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
    let r1u0: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(1, 0), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
    let l1d1: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(-1, 1), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
    let l0d1: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(0, 1), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
    let r1d1: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(1, 1), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);

    // Modified code for 5x5 neighborhood
    var liveNeighbors: i32 = 0;

    // Loop through the 5x5 neighborhood
    for (var dy: i32 = -2; dy <= 2; dy++) {
        for (var dx: i32 = -2; dx <= 2; dx++) {
            // Skip the center cell
            if (dx != 0 || dy != 0) {
                let neighborCell: vec4<f32> = textureLoad(r_color, clamp(uv + vec2<i32>(dx, dy), vec2<i32>(0), textureSize - vec2<i32>(1)), 0);
                liveNeighbors += i32(neighborCell.r);
            }
        }
    }

    var newCell: vec4<f32> = currentCell;
    if (currentCell.r > 0.0) {
        if (liveNeighbors < 4 || liveNeighbors > 9) {
            newCell = vec4<f32>(0.0);
        }
    } else {
        if (liveNeighbors == 10) {
            newCell = vec4<f32>(1.0);
        }
    }

    out.color0 = newCell;
    out.color1 = newCell;

    return out;
}