use std::{collections::HashMap, fs::read_to_string, path::PathBuf};

use anyhow::Result;
use naga::{
    AddressSpace, Module,
    back::{glsl, spv, wgsl as back_wgsl},
    front::wgsl as front_wgsl,
    proc::BoundsCheckPolicies,
    valid::{Capabilities, ModuleInfo, ValidationFlags, Validator},
};
use tracing::{debug, info, info_span};

use crate::{
    shading::{
        reflection::{
            ShaderAddressSpace, ShaderBlock, ShaderReflection, ShaderResource, ShaderResourceKind,
        },
        shader::{Shader, ShaderCode, ShaderOutput, ShaderStage},
    },
    util::types::BackendType,
};

#[derive(Clone)]
pub enum Source {
    Path(PathBuf),
    Source(String),
}

pub fn compile(source: Source, backend: BackendType) -> Result<(Shader, ShaderReflection)> {
    let span = info_span!(
        "Shader Compiler",
        backend = ?backend,
    );

    let _enter = span.enter();

    info!("Compiling shader");

    // Read source
    let shader_source = match source {
        Source::Path(path) => {
            debug!(
                path = %path.display(),
                "Reading shader source"
            );

            read_to_string(path)?
        }

        Source::Source(source) => {
            debug!(bytes = source.len(), "Using in-memory shader source");

            source
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
    let reflection = reflect(&module);

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

    // Compile
    let code = match backend {
        BackendType::OpenGL => compile_glsl(&module, &info)?,

        BackendType::Vulkan => compile_spirv(&module, &info)?,

        BackendType::WebGPU => compile_wgsl(&module, &info)?,
    };

    let shader = Shader {
        output: ShaderOutput { code, entry_points },
    };

    info!(
        stages = shader.output.code.len(),
        entry_points = shader.output.entry_points.len(),
        "Shader compilation complete"
    );

    Ok((shader, reflection))
}

fn reflect(module: &Module) -> ShaderReflection {
    let mut resources = HashMap::new();
    let mut blocks = HashMap::new();

    for (_, global) in module.global_variables.iter() {
        let Some(binding) = global.binding else {
            continue;
        };

        let (kind, address_space) = match global.space {
            AddressSpace::Uniform => (
                ShaderResourceKind::UniformBuffer,
                ShaderAddressSpace::Uniform,
            ),
            AddressSpace::Storage { .. } => (
                ShaderResourceKind::StorageBuffer,
                ShaderAddressSpace::Storage,
            ),

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

    ShaderReflection { resources, blocks }
}

fn compile_glsl(module: &Module, info: &ModuleInfo) -> Result<HashMap<ShaderStage, ShaderCode>> {
    let mut stages = HashMap::new();

    for entry_point in &module.entry_points {
        let Some(stage) = ShaderStage::try_from(entry_point.stage).ok() else {
            continue;
        };

        let span = info_span!(
            "compile_stage",
            stage = ?stage,
            entry_point = %entry_point.name,
            backend = "OpenGL",
        );

        let _enter = span.enter();

        info!("Compiling shader stage");

        let options = glsl::Options::default();

        let pipeline_options = glsl::PipelineOptions {
            shader_stage: entry_point.stage,
            entry_point: entry_point.name.clone(),
            multiview: None,
        };

        let mut output = String::new();

        let mut writer = glsl::Writer::new(
            &mut output,
            module,
            info,
            &options,
            &pipeline_options,
            BoundsCheckPolicies::default(),
        )?;

        writer.write()?;

        debug!(bytes = output.len(), "Generated GLSL");

        stages.insert(stage, ShaderCode::Text(output));

        info!("Shader stage compiled");
    }

    Ok(stages)
}

fn compile_spirv(module: &Module, info: &ModuleInfo) -> Result<HashMap<ShaderStage, ShaderCode>> {
    let mut stages = HashMap::new();

    let options = spv::Options::default();

    let mut words = Vec::<u32>::new();

    let mut writer = spv::Writer::new(&options)?;

    writer.write(module, info, None, &None, &mut words)?;

    let bytes: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();

    debug!(words = words.len(), bytes = bytes.len(), "Generated SPIR-V");

    // The SPIR-V module contains all entry points.
    // Each supported stage references the same binary.
    for entry_point in &module.entry_points {
        let Some(stage) = ShaderStage::try_from(entry_point.stage).ok() else {
            continue;
        };

        stages.insert(stage, ShaderCode::Binary(bytes.clone()));
    }

    Ok(stages)
}

fn compile_wgsl(module: &Module, info: &ModuleInfo) -> Result<HashMap<ShaderStage, ShaderCode>> {
    let mut output = String::new();

    let flags = back_wgsl::WriterFlags::all();

    let mut writer = back_wgsl::Writer::new(&mut output, flags);

    writer.write(module, info)?;

    debug!(bytes = output.len(), "Generated WGSL");

    // WGSL is a module containing all entry points.
    // Each supported stage references the same source.
    let mut stages = HashMap::new();

    for entry_point in &module.entry_points {
        let Some(stage) = ShaderStage::try_from(entry_point.stage).ok() else {
            continue;
        };

        stages.insert(stage, ShaderCode::Text(output.clone()));
    }

    Ok(stages)
}

impl TryFrom<naga::ShaderStage> for ShaderStage {
    type Error = ();

    fn try_from(stage: naga::ShaderStage) -> Result<Self, Self::Error> {
        match stage {
            naga::ShaderStage::Vertex => Ok(Self::Vertex),

            naga::ShaderStage::Fragment => Ok(Self::Fragment),

            naga::ShaderStage::Compute => Ok(Self::Compute),

            _ => Err(()),
        }
    }
}
