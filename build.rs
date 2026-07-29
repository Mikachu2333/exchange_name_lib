#[cfg(windows)]
use std::path::{Path, PathBuf};

fn main() {
    #[cfg(windows)]
    build_windows_resource();
}

#[cfg(windows)]
fn build_windows_resource() {
    assert!(
        !std::env::var("CARGO_CFG_TARGET_ENV").is_ok_and(|value| value.eq_ignore_ascii_case("gnu")),
        "Windows GNU targets are unsupported; use an MSVC target"
    );

    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR"));
    let resource = prepare_resource_files(&out_dir).expect("failed to prepare Windows resource");
    match embed_resource::compile(resource, embed_resource::NONE) {
        embed_resource::CompilationResult::NotAttempted(error)
        | embed_resource::CompilationResult::Failed(error) => panic!("{error}"),
        _ => {}
    }
}

#[cfg(windows)]
fn prepare_resource_files(out_dir: &Path) -> Result<String, std::io::Error> {
    let version = env!("CARGO_PKG_VERSION");
    let version_commas = version.replace('.', ",") + ",0";
    let header =
        format!("#define VERSION_INT  {version_commas}\n#define VERSION_STR  \"{version}\"\n");

    std::fs::write(out_dir.join("version.h"), header)?;
    let generated = out_dir.join("res.generated.rc");
    std::fs::copy("res.rc", &generated)?;
    Ok(generated.to_string_lossy().into_owned())
}
