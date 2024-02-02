use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::schema;

impl schema::Entry {
    pub fn group_layout_entry_impl(
        &self,
        binding: u32,
        visibility: proc_macro2::TokenStream,
    ) -> TokenStream2 {
        let binding_type = match self.variant {
            schema::EntryVariant::Texture2D { .. } => quote! {wgpu::BindingType::Texture {
                multisampled: false,
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
            }},
            schema::EntryVariant::Uniform { .. } => quote! {wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            }},
        };
        let layout_entry_const = quote! {
            const LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
                binding: #binding,
                visibility: #visibility,
                ty: #binding_type,
                count: None,
            };
        };
        let struct_ident = format_ident!("{}", &self.ty);
        match self.variant {
            schema::EntryVariant::Texture2D { width, height } => quote! {
                impl TextureProvider for #struct_ident {
                    #layout_entry_const
                    const WIDTH: u32 = #width;
                    const HEIGHT: u32 = #height;
                }
            },
            schema::EntryVariant::Uniform { size } => quote! {
                impl UniformProvider for #struct_ident {
                    #layout_entry_const
                    const SIZE: u64 = #size;
                }
            },
        }
    }
}

// struct MainShader {}
// impl MainShader {
//     fn group_0_layout_descriptor(&'static self) -> wgpu::BindGroupLayoutDescriptor {
//         wgpu::BindGroupLayoutDescriptor {
//             label: None,
//             entries: &[PreviousFrame::LAYOUT_ENTRY, Flags::LAYOUT_ENTRY],
//         }
//     }
// }

// struct TestShader {}
// impl TextureProvider for TestShader {
//     const LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry = wgpu::BindGroupLayoutEntry {
//         binding: 0u32,
//         visibility: wgpu::ShaderStages::FRAGMENT,
//         ty: wgpu::BindingType::Texture {
//             multisampled: false,
//             sample_type: wgpu::TextureSampleType::Float { filterable: false },
//             view_dimension: wgpu::TextureViewDimension::D2,
//         },
//         count: None,
//     };
//     fn allocate(&'static self) {}
// }

// pub fn pipeline_layout_descriptor_builder() -> TokenStream2 {
//     quote! {
//         pub fn pipeline_layout_descriptor(&'static self, device: &wgpu::Device) -> wgpu::PipelineLayoutDescriptor<'static> {
//             PipelineLayoutDescriptor {
//                 label: None,
//                 bind_group_layouts: &bind_group_layouts,
//                 push_constant_ranges: &[],
//             }
//         }
//     }
// }
