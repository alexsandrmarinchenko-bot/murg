fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=src/player_view.slint");
    println!("cargo:rerun-if-changed=Frame/assets");
    slint_build::compile("src/player_view.slint")?;
    Ok(())
}
