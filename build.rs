fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=Frame/main.slint");
    println!("cargo:rerun-if-changed=Frame/assets");
    slint_build::compile("Frame/main.slint")?;
    Ok(())
}
