// Puts the Lurt icon and version details into Lurt.exe. An exe with no name, publisher or
// version looks anonymous to Windows and to antivirus scanners.
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=assets/lurt.ico");

    let var = |k: &str| env::var(k).unwrap();
    let version = var("CARGO_PKG_VERSION");
    let numbers = format!(
        "{},{},{},0",
        var("CARGO_PKG_VERSION_MAJOR"),
        var("CARGO_PKG_VERSION_MINOR"),
        var("CARGO_PKG_VERSION_PATCH")
    );
    let description = var("CARGO_PKG_DESCRIPTION");
    let icon = var("CARGO_MANIFEST_DIR").replace('\\', "/") + "/assets/lurt.ico";

    let rc = format!(
        r#"1 ICON "{icon}"

1 VERSIONINFO
FILEVERSION {numbers}
PRODUCTVERSION {numbers}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "ladroxd"
      VALUE "FileDescription", "Lurt"
      VALUE "Comments", "{description}"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "lurt"
      VALUE "LegalCopyright", "Copyright (c) ladroxd"
      VALUE "OriginalFilename", "Lurt.exe"
      VALUE "ProductName", "Lurt"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    );
    let out = PathBuf::from(var("OUT_DIR")).join("lurt.rc");
    fs::write(&out, rc).unwrap();
    embed_resource::compile(&out, embed_resource::NONE).manifest_optional().unwrap();
}
