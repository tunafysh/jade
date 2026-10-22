use std::collections::HashMap;

use crate::shading::reflection::{ShaderReflection, ShaderResourceKind};

pub struct OpenGLReflection {
    pub resources: HashMap<String, OpenGLResource>,
}

impl From<ShaderReflection> for OpenGLReflection {
    fn from(reflection: ShaderReflection) -> Self {
        let mut resources = HashMap::new();

        for (name, resource) in reflection.resources {
            let kind = match resource.kind {
                ShaderResourceKind::UniformBuffer => OpenGLResourceKind::UniformBuffer,
                ShaderResourceKind::StorageBuffer => OpenGLResourceKind::StorageBuffer,
                ShaderResourceKind::Texture => OpenGLResourceKind::Texture,
                ShaderResourceKind::Sampler => OpenGLResourceKind::Sampler,
            };

            resources.insert(
                name,
                OpenGLResource {
                    group: resource.group,
                    binding: resource.binding,
                    kind,
                },
            );
        }

        OpenGLReflection { resources }
    }
}

pub struct OpenGLResource {
    pub group: u32,
    pub binding: u32,
    pub kind: OpenGLResourceKind,
}

pub enum OpenGLResourceKind {
    UniformBuffer,
    StorageBuffer,
    Texture,
    Sampler,
}
