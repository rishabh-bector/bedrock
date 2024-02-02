use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

mod gen;
mod parser;
mod schema;

use parser::*;
use schema::*;

#[proc_macro_attribute]
pub fn shader(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as syn::LitStr);
    let input = parse_macro_input!(item as DeriveInput);

    let mut parser = ShaderParser::new();
    let fields = if let syn::Data::Struct(data) = input.data {
        data.fields
    } else {
        unimplemented!();
    };
    for field in &fields {
        parser.process_field(field.clone());
    }
    let shader = Shader {
        path: attr.value(),
        vertex_buffers: parser.vertex_buffers,
        index_buffers: parser.index_buffers,
        bind_groups: parser.bind_groups,
        outputs: parser.outputs,
    };
    println!("parsed, generating...");

    // Code generation //

    let shader_ident = input.ident.clone();
    let shader_mod_ident = format_ident!("{}", transform_name(&input.ident.to_string()));

    let mut extra_gen = Vec::<TokenStream2>::new();
    let mut impl_gen = Vec::<TokenStream2>::new();
    let mut group_layout_cells = Vec::<TokenStream2>::new();
    let mut group_layout_cell_defs = Vec::<TokenStream2>::new();
    let mut group_layout_cell_idents = Vec::<TokenStream2>::new();
    let mut group_builders = Vec::<TokenStream2>::new();
    let mut draw_encoder_arg_groups = Vec::<TokenStream2>::new();

    for (group_index, group) in shader.bind_groups.iter().enumerate() {
        let visibility = match (group.vertex, group.fragment) {
            (true, false) => quote! { wgpu::ShaderStages::VERTEX },
            (false, true) => quote! { wgpu::ShaderStages::FRAGMENT},
            (true, true) => quote! { wgpu::ShaderStages::VERTEX_FRAGMENT },
            (false, false) => todo!(),
        };
        let (group_layout_entry_impls, group_layout_entry_constant_idents): (
            TokenStream2,
            TokenStream2,
        ) = group
            .entries
            .iter()
            .enumerate()
            .map(|(binding, entry)| {
                let entry_type = format_ident!("{}", entry.ty);
                (
                    entry.group_layout_entry_impl(binding as u32, visibility.clone()),
                    quote! { #entry_type::LAYOUT_ENTRY, },
                )
            })
            .unzip();

        let descriptor_ident = format_ident!("group_{}_layout_descriptor", group_index);
        let group_layout_descriptor_static = quote! {
            pub fn #descriptor_ident() -> wgpu::BindGroupLayoutDescriptor<'static> {
                wgpu::BindGroupLayoutDescriptor{
                    label: None,
                    entries: &[
                        #group_layout_entry_constant_idents
                    ]
                }
            }
        };

        let layout_ident = format_ident!("group_{}_layout", group_index);
        let group_layout_builder = quote! {
            pub fn #layout_ident(device: &wgpu::Device) -> wgpu::BindGroupLayout {
                device.create_bind_group_layout(&Self::#descriptor_ident())
            }
        };

        extra_gen.push(quote! {
            #group_layout_entry_impls
        });
        impl_gen.push(quote! {
            #group_layout_descriptor_static
            #group_layout_builder
        });

        let static_ident = format_ident!("GROUP_{}_LAYOUT", group_index);
        group_layout_cells.push(quote! {
            pub static #static_ident: OnceLock<wgpu::BindGroupLayout> = OnceLock::new();
        });

        group_layout_cell_defs.push(quote! {
            let #layout_ident = #static_ident.get_or_init(|| #shader_ident::#layout_ident(device));
        });

        group_layout_cell_idents.push(quote! {
            #layout_ident,
        });

        let mut group_builder_args = Vec::<TokenStream2>::new();
        let mut group_builder_binding_resources = Vec::<TokenStream2>::new();
        for (entry_index, entry) in group.entries.iter().enumerate() {
            let arg_ident = format_ident!("entry_{entry_index}");
            match entry.variant {
                EntryVariant::Texture2D { .. } => {
                    group_builder_args.push(quote! {
                        #arg_ident: &ShaderTexture,
                    });
                    group_builder_binding_resources.push(quote! {
                        wgpu::BindingResource::TextureView(&#arg_ident.view)
                    });
                }
                EntryVariant::Uniform { .. } => {
                    group_builder_args.push(quote! {
                        #arg_ident: &ShaderUniform,
                    });
                    group_builder_binding_resources.push(quote! {
                        wgpu::BindingResource::Buffer(#arg_ident.buffer.as_entire_buffer_binding())
                    });
                }
            }
        }
        let group_builder_args = group_builder_args.into_iter().collect::<TokenStream2>();
        let group_builder_binding_entries = group_builder_binding_resources
            .into_iter()
            .enumerate()
            .map(|(binding, resource)| {
                let binding = binding as u32;
                quote! {
                    wgpu::BindGroupEntry {
                        binding: #binding,
                        resource: #resource,
                    },
                }
            })
            .collect::<TokenStream2>();

        let group_builder_ident = format_ident!("group_{group_index}");
        group_builders.push(quote! {
            pub fn #group_builder_ident(device: &wgpu::Device, #group_builder_args) -> wgpu::BindGroup {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: #static_ident.get().unwrap(),
                    entries: &[#group_builder_binding_entries],
                })
            }
        });

        draw_encoder_arg_groups.push(quote! {
            #group_builder_ident: &'a wgpu::BindGroup,
        });
    }

    let mut draw_encoder_arg_targets = Vec::<TokenStream2>::new();
    let mut draw_encoder_color_targets = Vec::<TokenStream2>::new();
    for (output_index, _) in shader.outputs.iter().enumerate() {
        let output_ident = format_ident!("output_{output_index}");
        draw_encoder_arg_targets.push(quote! {
            #output_ident: &'a wgpu::TextureView,
        });
        draw_encoder_color_targets.push(quote! {
            Some(wgpu::RenderPassColorAttachment {
                view: #output_ident,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            }),
        })
    }

    let mut render_pass_binds = Vec::<TokenStream2>::new();
    for (group_index, _) in shader.bind_groups.iter().enumerate() {
        let group_ident = format_ident!("group_{}", group_index);
        let group_index = group_index as u32;
        render_pass_binds.push(quote! {
            render_pass.set_bind_group(#group_index, #group_ident, &[]);
        });
    }

    let group_layout_cells = group_layout_cells.into_iter().collect::<TokenStream2>();
    let group_layout_cell_defs = group_layout_cell_defs.into_iter().collect::<TokenStream2>();
    let group_layout_cell_idents = group_layout_cell_idents
        .into_iter()
        .collect::<TokenStream2>();
    let extra_gen = extra_gen.into_iter().collect::<TokenStream2>();
    let impl_gen = impl_gen.into_iter().collect::<TokenStream2>();
    let group_builders = group_builders.into_iter().collect::<TokenStream2>();
    let draw_encoder_arg_groups = draw_encoder_arg_groups
        .into_iter()
        .collect::<TokenStream2>();
    let draw_encoder_arg_targets = draw_encoder_arg_targets
        .into_iter()
        .collect::<TokenStream2>();
    let draw_encoder_color_targets = draw_encoder_color_targets
        .into_iter()
        .collect::<TokenStream2>();
    let render_pass_binds = render_pass_binds.into_iter().collect::<TokenStream2>();

    let expanded = quote! {
        #extra_gen
        struct #shader_ident {}
        impl #shader_ident {
            #impl_gen
        }
        mod #shader_mod_ident {
            use std::{sync::OnceLock, borrow::Cow, ops::Range};

            use bedrock_lib::{ShaderTexture, ShaderUniform, driver};

            use super::#shader_ident;

            #group_layout_cells

            pub static GROUP_LAYOUTS: OnceLock<Vec<&'static wgpu::BindGroupLayout>> = OnceLock::new();
            pub static PIPELINE_LAYOUT_DESCRIPTOR: OnceLock<wgpu::PipelineLayoutDescriptor> = OnceLock::new();
            pub static PIPELINE: OnceLock<wgpu::RenderPipeline> = OnceLock::new();

            #group_builders

            pub fn pipeline(device: &wgpu::Device) -> &wgpu::RenderPipeline {
                #group_layout_cell_defs
                let group_layouts = GROUP_LAYOUTS.get_or_init(|| vec![#group_layout_cell_idents]);

                let pipeline_layout_descriptor = PIPELINE_LAYOUT_DESCRIPTOR.get_or_init(||
                    wgpu::PipelineLayoutDescriptor {
                            label: None,
                            bind_group_layouts: group_layouts,
                            push_constant_ranges: &[],
                    });
                let pipeline_layout = device.create_pipeline_layout(pipeline_layout_descriptor);

                let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
                });

                PIPELINE.get_or_init(|| {
                    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                        label: None,
                        layout: Some(&pipeline_layout),
                        vertex: wgpu::VertexState {
                            module: &shader,
                            entry_point: "vs_main",
                            buffers: &[],
                        },
                        fragment: Some(wgpu::FragmentState {
                            module: &shader,
                            entry_point: "fs_main",
                            targets: &[Some(wgpu::ColorTargetState::from(*(driver::SWAP_CHAIN_FORMAT.get().unwrap()))), Some(wgpu::ColorTargetState::from(*(driver::SWAP_CHAIN_FORMAT.get().unwrap())))]
                        }),
                        primitive: wgpu::PrimitiveState::default(),
                        depth_stencil: None,
                        multisample: wgpu::MultisampleState::default(),
                        multiview: None,
                    })
                })
            }

            pub fn draw_encoder<'a>(device: &wgpu::Device, vertices: Range<u32>, instances: Range<u32>, #draw_encoder_arg_groups #draw_encoder_arg_targets) -> wgpu::CommandEncoder {
                let pipeline = PIPELINE.get().unwrap();
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: None,
                });
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[#draw_encoder_color_targets],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                render_pass.set_pipeline(pipeline);
                #render_pass_binds
                render_pass.draw(vertices, instances);

                drop(render_pass);
                encoder
            }
        }
    };

    TokenStream::from(expanded)
}

// -----------------------------------------------------------------------------------------------
// This should generate:
//
// Pipeline
// - static for pipeline descriptor
// - builder for pipeline
// Requires:
// - Bind Groups (Uniforms + Textures)
//      - constants for each bind group entry layout
//      - statics for each bind group layout descriptor
//      - builders for each bind group layout
//      - builders for each bind group?
// - Vertex/Index Buffers
//      - constants for each buffer descriptor
//      - builders for each buffer
// - Targets

fn transform_name(name: &str) -> String {
    let mut transformed_name = String::new();
    let mut prev_char = '_';

    for c in name.chars() {
        if c.is_uppercase() {
            if prev_char != '_' {
                transformed_name.push('_');
            }
            transformed_name.push(c.to_lowercase().next().unwrap());
        } else {
            transformed_name.push(c);
        }
        prev_char = c;
    }

    transformed_name
}
