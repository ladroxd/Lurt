// Puts the Lurt icon (and later version info) into lurt.exe.
fn main() {
    println!("cargo:rerun-if-changed=assets/lurt.rc");
    println!("cargo:rerun-if-changed=assets/lurt.ico");
    embed_resource::compile("assets/lurt.rc", embed_resource::NONE).manifest_optional().unwrap();
}
