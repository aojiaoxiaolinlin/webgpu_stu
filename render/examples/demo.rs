use std::vec;

use render::{App, SpecialRenderPipeline, render::mesh::Vertex};
use wgpu::{MultisampleState, PrimitiveState, VertexState, util::DeviceExt};

fn main() {
    App::run(VertexRenderPipeline);
}

struct VertexRenderPipeline;

impl SpecialRenderPipeline for VertexRenderPipeline {
    fn special_render_pipeline(
        &self,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../assets/wgsls/demo.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        render_pipeline
    }

    fn draw(
        &self,
        mut render_pass: wgpu::RenderPass,
        render_pipeline: &wgpu::RenderPipeline,
        device: &wgpu::Device,
    ) {
        let vertices_one = vec![
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ];

        let vertices_slice: &[u8] = bytemuck::cast_slice(&vertices_one);

        let offset = vertices_slice.len() as u64;

        let vertices_two = vec![
            Vertex {
                position: [0.0, 0.5, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
            Vertex {
                position: [1.0, 0.5, 0.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
        ];

        let vertices_slice_two: &[u8] = bytemuck::cast_slice(&vertices_two);

        let indices_one = bytemuck::cast_slice(&[0, 1, 2]);
        let indices_two = bytemuck::cast_slice(&[3, 4, 5]);

        let indices_offset = indices_one.len() as u64;

        let mut vertices = Vec::new();
        vertices.extend_from_slice(vertices_slice);
        vertices.extend_from_slice(vertices_slice_two);
        let mut indices = Vec::new();
        indices.extend_from_slice(indices_one);
        indices.extend_from_slice(indices_two);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertices.as_slice(),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: indices.as_slice(),
            usage: wgpu::BufferUsages::INDEX,
        });

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..offset));
        render_pass.set_index_buffer(
            index_buffer.slice(..indices_offset),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..3 as u32, 0, 0..1);

        render_pass.set_vertex_buffer(1, vertex_buffer.slice(offset..));
        render_pass.set_index_buffer(
            index_buffer.slice(indices_offset..),
            wgpu::IndexFormat::Uint32,
        );
        render_pass.draw_indexed(0..3 as u32, 0, 0..1);

        // render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        // render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        // render_pass.draw_indexed(0..3 as u32, 0, 0..1);
        // render_pass.draw_indexed(3..6 as u32, 0, 0..1);
    }
}
