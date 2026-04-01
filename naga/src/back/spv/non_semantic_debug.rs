//! Constants for the [`NonSemantic.Shader.DebugInfo.100`][spec] extended instruction set.
//!
//! These opcodes are used with `OpExtInst` to emit rich debug information
//! that tools like NVIDIA Nsight and RenderDoc can consume.
//!
//! [spec]: https://github.com/KhronosGroup/SPIRV-Headers/blob/main/include/spirv/unified1/NonSemanticShaderDebugInfo100.h

// Not all opcodes and flags are used yet — they are defined here for
// completeness and future use.
#![allow(dead_code)]

/// Instruction opcodes for `NonSemantic.Shader.DebugInfo.100`.
///
/// Each constant corresponds to the `instruction` operand of an `OpExtInst`
/// referencing the `"NonSemantic.Shader.DebugInfo.100"` import.
pub const DEBUG_INFO_NONE: u32 = 0;
pub const DEBUG_COMPILATION_UNIT: u32 = 1;
pub const DEBUG_TYPE_FUNCTION: u32 = 8;
pub const DEBUG_FUNCTION: u32 = 20;
pub const DEBUG_LEXICAL_BLOCK: u32 = 21;
pub const DEBUG_SCOPE: u32 = 23;
pub const DEBUG_NO_SCOPE: u32 = 24;
pub const DEBUG_SOURCE: u32 = 35;
pub const DEBUG_FUNCTION_DEFINITION: u32 = 101;
pub const DEBUG_LINE: u32 = 103;
pub const DEBUG_NO_LINE: u32 = 104;
pub const DEBUG_ENTRY_POINT: u32 = 107;

/// `DebugInfoFlags` values used in `DebugFunction` and related instructions.
pub const FLAG_NONE: u32 = 0;
pub const FLAG_IS_PUBLIC: u32 = 3;
pub const FLAG_IS_DEFINITION: u32 = 8;
