#[derive(Debug)]
pub struct Group {
    pub ident: String,
    pub vertex: bool,
    pub fragment: bool,
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub ident: String,
    pub ty: String,
    pub variant: EntryVariant,
}

#[derive(Debug)]
pub enum EntryVariant {
    Texture2D { width: u32, height: u32 },
    Uniform { size: u64 },
}

#[derive(Debug)]
pub struct Shader {
    pub path: String,
    pub vertex_buffers: Vec<Vertex>,
    pub index_buffers: Vec<Index>,
    pub bind_groups: Vec<Group>,
    pub outputs: Vec<Output>,
}

#[derive(Debug)]
pub struct Index {}

#[derive(Debug)]
pub struct Vertex {}

#[derive(Debug)]
pub struct Output {}
