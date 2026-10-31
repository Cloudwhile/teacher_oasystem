fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let manifest = std::path::Path::new("app.manifest")
            .canonicalize()
            .expect("desktop manifest");
        println!("cargo:rustc-link-arg-bin=teacher_oa_desktop=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=teacher_oa_desktop=/MANIFESTINPUT:{}",
            manifest.display()
        );
        println!("cargo:rerun-if-changed=app.manifest");
    }
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent-light".into()),
    )
    .expect("compile desktop interface");
}
