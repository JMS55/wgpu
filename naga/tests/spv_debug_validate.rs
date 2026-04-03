//! Validation test: generate SPIR-V with NonSemantic.Shader.DebugInfo.100 and run spirv-val.

use std::process::Command;

const WGSL: &str = r#"
@compute @workgroup_size(1)
fn main() {
    var x: f32 = 1.0;
    var y: vec2<f32> = vec2<f32>(x, 2.0);
    var i: i32 = 0;
}
"#;

#[test]
fn validate_nonsemantic_debug_info() {
    let module = naga::front::wgsl::parse_str(WGSL).expect("WGSL parse failed");
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .expect("Naga validation failed");

    let debug_info = naga::back::spv::DebugInfo {
        source_code: WGSL,
        file_name: "test.wgsl",
        language: naga::back::spv::SourceLanguage::GLSL,
    };
    let options = naga::back::spv::Options {
        flags: naga::back::spv::WriterFlags::DEBUG,
        debug_info: Some(debug_info),
        ..Default::default()
    };
    let spv = naga::back::spv::write_vec(&module, &info, &options, None)
        .expect("SPIR-V generation failed");

    // Convert to bytes
    let spv_bytes: Vec<u8> = spv.iter().flat_map(|w| w.to_le_bytes()).collect();

    // Write to a temp file
    let spv_path = std::env::temp_dir().join("naga_debug_info_test.spv");
    std::fs::write(&spv_path, &spv_bytes).expect("Failed to write SPIR-V binary");

    // Look for spirv-val in common locations (including Windows Vulkan SDK).
    let spirv_val_candidates = [
        "C:/VulkanSDK/1.4.341.1/Bin/spirv-val.exe",
        "C:/VulkanSDK/1.3.296.0/Bin/spirv-val.exe",
        "/usr/bin/spirv-val",
        "/usr/local/bin/spirv-val",
        "spirv-val",
    ];
    let spirv_val = spirv_val_candidates
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .or_else(|| {
            // Also try finding it via PATH
            spirv_val_candidates.last()
        });
    let Some(spirv_val) = spirv_val else {
        eprintln!("spirv-val not found; skipping validation");
        return;
    };

    let result = Command::new(spirv_val)
        .arg("--target-env")
        .arg("vulkan1.1")
        .arg(&spv_path)
        .output()
        .expect("Failed to run spirv-val");

    if !result.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&result.stdout));
    }
    if !result.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&result.stderr));
    }
    assert!(
        result.status.success(),
        "spirv-val failed with exit code {:?}",
        result.status.code()
    );
}
