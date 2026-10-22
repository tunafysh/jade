use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow};
use glow::{Context, HasContext};
use tracing::error;

use crate::shading::{
    reflection::{ShaderReflection, ShaderResourceKind},
    shader::{Shader, ShaderCode, ShaderStage},
};

pub struct OpenGLUniformBuffer {
    pub buffer: glow::NativeBuffer,
    pub binding: u32,
    pub size: usize,
}
pub struct OpenGLShader {
    pub program: glow::NativeProgram,
    pub uniform_buffers: HashMap<String, OpenGLUniformBuffer>,
    pub gl: Arc<Context>,
}

impl OpenGLShader {
    pub fn new(gl: Arc<Context>, shader: &Shader, reflection: ShaderReflection) -> Result<Self> {
        let program = unsafe {
            gl.create_program().map_err(|error| {
                error!(
                    error = %error,
                    "Failed to create OpenGL shader program"
                );

                anyhow!("Failed to create OpenGL shader program: {error}")
            })?
        };

        let vertex = match Self::compile_stage(
            &gl,
            program,
            shader,
            ShaderStage::Vertex,
            glow::VERTEX_SHADER,
        ) {
            Ok(shader) => shader,

            Err(error) => {
                unsafe {
                    gl.delete_program(program);
                }

                return Err(error);
            }
        };

        let fragment = match Self::compile_stage(
            &gl,
            program,
            shader,
            ShaderStage::Fragment,
            glow::FRAGMENT_SHADER,
        ) {
            Ok(shader) => shader,

            Err(error) => {
                unsafe {
                    gl.delete_shader(vertex);
                    gl.delete_program(program);
                }

                return Err(error);
            }
        };

        unsafe {
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);

                error!(
                    error = %log,
                    "Failed to link OpenGL shader program"
                );

                gl.delete_shader(vertex);
                gl.delete_shader(fragment);
                gl.delete_program(program);

                return Err(anyhow!("Failed to link OpenGL shader program: {log}"));
            }

            gl.delete_shader(vertex);
            gl.delete_shader(fragment);
        }

        let uniform_buffers = Self::create_uniform_buffers(&gl, program, &reflection)?;

        Ok(Self {
            program,
            uniform_buffers,
            gl,
        })
    }

    fn create_uniform_buffers(
        gl: &Context,
        _program: glow::NativeProgram,
        reflection: &ShaderReflection,
    ) -> Result<HashMap<String, OpenGLUniformBuffer>> {
        let mut map = HashMap::new();

        for (name, resource) in &reflection.resources {
            if resource.kind != ShaderResourceKind::UniformBuffer {
                continue;
            }

            let block = reflection
                .blocks
                .get(name)
                .ok_or_else(|| anyhow!("Uniform buffer '{name}' has no reflected block"))?;

            let buffer = unsafe {
                gl.create_buffer().map_err(|error| {
                    error!(
                        resource = %name,
                        error = %error,
                        "Failed to create OpenGL uniform buffer"
                    );

                    anyhow!("Failed to create OpenGL uniform buffer '{name}': {error}")
                })?
            };

            unsafe {
                gl.bind_buffer(glow::UNIFORM_BUFFER, Some(buffer));

                gl.buffer_data_size(glow::UNIFORM_BUFFER, block.size as i32, glow::DYNAMIC_DRAW);

                gl.bind_buffer_base(glow::UNIFORM_BUFFER, resource.binding, Some(buffer));

                gl.bind_buffer(glow::UNIFORM_BUFFER, None);
            }

            map.insert(
                name.clone(),
                OpenGLUniformBuffer {
                    buffer,
                    binding: resource.binding,
                    size: block.size as usize,
                },
            );
        }

        Ok(map)
    }

    fn compile_stage(
        gl: &Context,
        program: glow::NativeProgram,
        shader: &Shader,
        stage: ShaderStage,
        shader_type: u32,
    ) -> Result<glow::NativeShader> {
        let code = shader
            .output
            .code
            .get(&stage)
            .ok_or_else(|| anyhow!("Shader does not contain {stage:?} stage"))?;

        let source = match code {
            ShaderCode::Text(source) => source,

            ShaderCode::Binary(_) => {
                return Err(anyhow!(
                    "OpenGL shader stage {stage:?} contains binary code"
                ));
            }
        };

        unsafe {
            let shader_object = gl.create_shader(shader_type).map_err(|error| {
                error!(
                    stage = ?stage,
                    error = %error,
                    "Failed to create OpenGL shader object"
                );

                anyhow!("Failed to create OpenGL shader object: {error}")
            })?;

            gl.shader_source(shader_object, source);

            gl.compile_shader(shader_object);

            if !gl.get_shader_compile_status(shader_object) {
                let log = gl.get_shader_info_log(shader_object);

                error!(
                    stage = ?stage,
                    error = %log,
                    "Failed to compile OpenGL shader"
                );

                gl.delete_shader(shader_object);

                return Err(anyhow!("Failed to compile OpenGL {stage:?} shader: {log}"));
            }

            gl.attach_shader(program, shader_object);

            Ok(shader_object)
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }
}

impl Drop for OpenGLShader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}
