use std::path::Path;

#[test]
fn test_naga_validate_quad_alpha_erosion() {
    let vs_path = "assets/shaders/hlsl/particlesystem/quad/vs/ALPHA_EROSION.spv";
    let ps_path = "assets/shaders/hlsl/particlesystem/quad/ps/ALPHA_EROSION.spv";

    validate_spv(vs_path);
    validate_spv(ps_path);
}

#[test]
fn test_naga_validate_all_extracted_spvs() {
    let base_dir = Path::new("assets/shaders/hlsl");
    let mut count = 0usize;

    fn visit_dir(dir: &Path, count: &mut usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, count);
                } else if path.extension().and_then(|s| s.to_str()) == Some("spv") {
                    validate_spv(&path);
                    *count += 1;
                }
            }
        }
    }

    visit_dir(base_dir, &mut count);
    println!("Validated {count} SPIR-V shaders through Naga successfully!");
    assert!(count > 0, "No SPV files found to validate!");
}

fn validate_spv(path: impl AsRef<Path>) {
    let path = path.as_ref();
    let bytes =
        std::fs::read(path).unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    let options = naga::front::spv::Options::default();
    let module = naga::front::spv::parse_u8_slice(&bytes, &options)
        .unwrap_or_else(|e| panic!("Failed to parse SPIR-V in {}: {e}", path.display()));

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    );

    validator
        .validate(&module)
        .unwrap_or_else(|e| panic!("Naga validation failed for {}: {e:#?}", path.display()));
}
