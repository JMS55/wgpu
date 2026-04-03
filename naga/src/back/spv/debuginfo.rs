use spirv::Word;

/// Opcodes for the `NonSemantic.Shader.DebugInfo.100` extended instruction set.
///
/// <https://github.khronos.org/SPIRV-Registry/nonsemantic/NonSemantic.Shader.DebugInfo.html>
pub(super) mod opcodes {
    pub const DEBUG_INFO_NONE: u32 = 0;
    pub const DEBUG_COMPILATION_UNIT: u32 = 1;
    pub const DEBUG_TYPE_BASIC: u32 = 2;
    pub const DEBUG_TYPE_VECTOR: u32 = 6;
    pub const DEBUG_TYPE_FUNCTION: u32 = 8;
    pub const DEBUG_FUNCTION: u32 = 20;
    pub const DEBUG_SCOPE: u32 = 23;
    pub const DEBUG_LOCAL_VARIABLE: u32 = 26;
    pub const DEBUG_DECLARE: u32 = 28;
    pub const DEBUG_EXPRESSION: u32 = 31;
    pub const DEBUG_SOURCE: u32 = 35;
    pub const DEBUG_FUNCTION_DEFINITION: u32 = 101;
    #[allow(dead_code)]
    pub const DEBUG_LINE: u32 = 103;
    pub const DEBUG_ENTRY_POINT: u32 = 107;
}

/// `SourceLanguage` values for `DebugCompilationUnit` in `NonSemantic.Shader.DebugInfo.100`.
///
/// These differ from the `SourceLanguage` values used in `OpSource`.
pub(super) mod source_language {
    pub const GLSL: u32 = 2;
}

/// `DebugBaseTypeAttributeEncoding` values for `DebugTypeBasic`.
pub(super) mod encoding {
    pub const BOOLEAN: u32 = 2;
    pub const FLOAT: u32 = 3;
    pub const SIGNED: u32 = 4;
    pub const UNSIGNED: u32 = 6;
}

/// `DebugInfoFlags` bit values.
pub(super) mod flags {
    pub const IS_DEFINITION: u32 = 0x0008;
}

/// State for the `NonSemantic.Shader.DebugInfo.100` extended instruction set.
///
/// The `OpExtInstImport` for this extension is registered in the module's
/// `ext_inst_imports` section.  All other `DebugXxx` instructions are emitted
/// into function bodies (prelude blocks), so that rspirv and other parsers that
/// don't support non-semantic instructions in the global-declarations section
/// can still consume the output.
pub(super) struct NonSemanticShaderDebugInfo {
    /// The `OpExtInstImport` result ID for `NonSemantic.Shader.DebugInfo.100`.
    pub ext_id: Word,
}
