use std::path::PathBuf;

use anyhow::Result;

pub struct ShaderCompiler {
    global_session: slang::GlobalSession,
    search_path: PathBuf,
}

impl ShaderCompiler {
    pub fn new(search_path: impl Into<PathBuf>) -> Result<Self> {
        let global_session = slang::GlobalSession::new()
            .map_err(|e| anyhow::anyhow!("Failed to create Slang global session: {e:?}"))?;

        Ok(Self {
            global_session,
            search_path: search_path.into(),
        })
    }
}