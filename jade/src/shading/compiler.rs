use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use anyhow::{Result, anyhow};
use naga::{
    AddressSpace, Module, ShaderStage as NagaShaderStage,
    back::wgsl as back_wgsl,
    front::wgsl as front_wgsl,
    valid::{Capabilities, ModuleInfo, ValidationFlags, Validator},
};
use tracing::{debug, info, info_span};

use crate::shading::{
    reflection::{
        ShaderBlock, ShaderEntryPoint, ShaderField, ShaderReflection, ShaderResource,
        ShaderResourceKind,
    },
    shader::{Shader, ShaderOutput, ShaderStage},
};

#[derive(Clone, Debug)]
pub struct Snippet {
    pub name: String, //it was necessary dont ask
    pub source: String,
}

#[derive(Clone)]
pub enum ShaderSource {
    Path(PathBuf),
    Snippet(Snippet),
}

pub fn compile(source: ShaderSource) -> Result<(Shader, ShaderReflection)> {
    let span = info_span!("Shader Compiler",);

    let _enter = span.enter();

    info!("Compiling shader");

    // Read source
    let (shader_name, shader_source) = match source {
        ShaderSource::Path(path) => {
            debug!(
                path = %path.display(),
                "Reading shader source"
            );

            let name: String = path
                .file_name()
                .ok_or(anyhow!("How the hell did you even do this?!"))?
                .to_str()
                .ok_or(anyhow!("Please name the file better :/"))?
                .to_string();

            (name, read_to_string(path)?)
        }

        ShaderSource::Snippet(snippet) => {
            debug!(
                name = snippet.name,
                bytes = snippet.source.len(),
                "Using in-memory shader source"
            );

            (snippet.name, snippet.source)
        }
    };

    // Parse
    debug!("Parsing WGSL");

    let module = front_wgsl::parse_str(&shader_source)?;

    // Validate
    debug!("Validating shader");

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());

    let info = validator.validate(&module)?;

    info!(
        entry_points = module.entry_points.len(),
        globals = module.global_variables.len(),
        "Shader validated"
    );

    // Reflection
    let reflection = reflect(&module)?;

    info!(
        resources = reflection.resources.len(),
        "Shader reflection complete"
    );

    // Entry points
    let mut entry_points = HashMap::new();

    for entry_point in &module.entry_points {
        let Some(stage) = ShaderStage::try_from(entry_point.stage).ok() else {
            debug!(
                entry_point = %entry_point.name,
                stage = ?entry_point.stage,
                "Skipping unsupported shader stage"
            );

            continue;
        };

        debug!(
            stage = ?stage,
            entry_point = %entry_point.name,
            "Found shader entry point"
        );

        entry_points.insert(stage, entry_point.name.clone());
    }

    let emitted_shader = emit_wgsl(&module, &info)?;

    let shader = Shader {
        name: shader_name,
        output: ShaderOutput {
            source: emitted_shader,
            entry_points,
        },
    };

    info!(
        stages = shader.output.source.len(),
        entry_points = shader.output.entry_points.len(),
        "Shader compilation complete"
    );

    Ok((shader, reflection))
}

fn reflect(module: &Module) -> Result<ShaderReflection> {
    let mut resources = HashMap::new();
    let mut blocks = HashMap::new();
    let mut entrypoints = Vec::new();

    for (_, global) in module.global_variables.iter() {
        let Some(binding) = global.binding else {
            continue;
        };

        let (kind, address_space) = match global.space {
            AddressSpace::Uniform => (ShaderResourceKind::UniformBuffer, AddressSpace::Uniform),
            AddressSpace::Storage { .. } => (ShaderResourceKind::StorageBuffer, global.space),

            _ => continue,
        };

        let Some(name) = &global.name else {
            continue;
        };

        debug!(
            parameter = %name,
            group = binding.group,
            binding = binding.binding,
            kind = ?kind,
            "Reflected shader parameter"
        );

        resources.insert(
            name.clone(),
            ShaderResource {
                group: binding.group,
                binding: binding.binding,
                kind,
            },
        );

        blocks.insert(
            name.clone(),
            ShaderBlock {
                address_space,
                size: 0,
                fields: Vec::new(),
            },
        );
    }

    for entry in &module.entry_points {
        let mut inputs = Vec::new();

        for input in &entry.function.arguments {
            let ty = &module.types[input.ty];
            let field = ShaderField {
                name: input.name.clone(),
                ty: ty.inner.clone(),
                binding: input.binding.clone(),
            };

            inputs.push(field);
        }
        let parsed = ShaderEntryPoint {
            stage: ShaderStage::try_from(entry.stage)?,
            inputs,
            name: entry.name.clone(),
        };
        entrypoints.push(parsed);
    }

    Ok(ShaderReflection {
        resources,
        blocks,
        entry_points: entrypoints,
    })
}

fn emit_wgsl(module: &Module, info: &ModuleInfo) -> Result<String> {
    let mut output = String::new();

    let flags = back_wgsl::WriterFlags::all();
    let mut writer = back_wgsl::Writer::new(&mut output, flags);

    writer.write(module, info)?;

    debug!(bytes = output.len(), "Emitted WGSL");

    Ok(output)
}

impl TryFrom<NagaShaderStage> for ShaderStage {
    type Error = anyhow::Error;

    fn try_from(stage: NagaShaderStage) -> Result<Self, Self::Error> {
        match stage {
            NagaShaderStage::Vertex => Ok(Self::Vertex),

            NagaShaderStage::Fragment => Ok(Self::Fragment),

            NagaShaderStage::Compute => Ok(Self::Compute),

            _ => Err(anyhow!("This stage is not supported yet probs")),
        }
    }
}
