// THIS SHOULD ALLOW A PIPELINE TO BE BUILT:
// let pipeline = MainShader::build_pipeline(device);

#[derive(Debug, serde::Deserialize)]
struct Shader {
    source: String,
    vertex: Vertex,
    fragment: Fragment,
}

#[derive(Debug, serde::Deserialize)]
struct Vertex {
    main: String,
    buffers: Vec<Buffer>,
    inputs: Vec<Input>,
}

#[derive(Debug, serde::Deserialize)]
struct Fragment {
    main: String,
    inputs: Vec<Input>,
    targets: u32,
}

#[derive(Debug, serde::Deserialize)]
struct Input {
    textures: Vec<TextureInput>,
    uniforms: Vec<UniformInput>,
}

#[derive(Debug, serde::Deserialize)]
struct TextureInput {
    width: u32,
    height: u32,
}

#[derive(Debug, serde::Deserialize)]
struct UniformInput {
    size: u32,
}

#[derive(Debug, serde::Deserialize)]
struct Buffer {
    size: u64,
}

// --------------------------------------------------------------------------------
