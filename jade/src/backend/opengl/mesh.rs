use std::sync::Arc;

use anyhow::{Result, anyhow};
use glow::{Context, HasContext, LINES, NativeBuffer, NativeVertexArray, POINTS, TRIANGLES};

use crate::graphics::{
    mesh::{
        Mesh,
        Primitive::{Lines, Points, Triangles},
    },
    vertex::CVertex,
};

pub struct OpenGLMesh {
    vao: NativeVertexArray,
    vbo: NativeBuffer,
    ebo: NativeBuffer,
    draw_mode: u32,
    index_count: i32,
    gl: Arc<Context>,
}

impl OpenGLMesh {
    pub fn new(gl: Arc<Context>, mesh: &Mesh) -> Result<Self> {
        let vao = unsafe {
            gl.create_vertex_array()
                .map_err(|e| anyhow!("Failed to create VAO buffer: {}", e))?
        };

        let vbo = unsafe {
            gl.create_buffer()
                .map_err(|e| anyhow!("Failed to create VBO buffer: {}", e))?
        };

        let ebo = unsafe {
            gl.create_buffer()
                .map_err(|e| anyhow!("Failed to create EBO buffer: {}", e))?
        };

        let index_count = mesh.indices.len() as i32;

        let draw_mode = match mesh.primitive {
            Points => POINTS,
            Lines => LINES,
            Triangles => TRIANGLES,
        };

        let vertices: Vec<CVertex> = mesh
            .vertices
            .clone()
            .into_iter()
            .map(CVertex::from)
            .collect();

        let indices = mesh.indices.clone();

        unsafe {
            gl.bind_vertex_array(Some(vao));

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&vertices),
                glow::STATIC_DRAW,
            );
        }

        let stride = std::mem::size_of::<CVertex>() as i32;

        let position_offset = std::mem::offset_of!(CVertex, position) as i32;
        let normal_offset = std::mem::offset_of!(CVertex, normal) as i32;
        let uv_offset = std::mem::offset_of!(CVertex, uv) as i32;

        unsafe {
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, position_offset);

            gl.enable_vertex_attrib_array(1);
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, normal_offset);

            gl.enable_vertex_attrib_array(2);
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, uv_offset);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));

            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(&indices),
                glow::STATIC_DRAW,
            );

            gl.bind_vertex_array(None);
        }

        Ok(Self {
            vao,
            vbo,
            ebo,
            draw_mode,
            index_count,
            gl,
        })
    }

    pub fn draw(&self) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vao));

            self.gl
                .draw_elements(self.draw_mode, self.index_count, glow::UNSIGNED_INT, 0);

            self.gl.bind_vertex_array(None);
        }
    }
}

impl Drop for OpenGLMesh {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_buffer(self.ebo);
        }
    }
}
