//! Windows file dialogs: pick a folder, or choose where to save a file.

use windows::{
    Win32::{
        Foundation::HWND,
        System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree},
        UI::Shell::{
            FOS_FORCEFILESYSTEM, FOS_OVERWRITEPROMPT, FOS_PICKFOLDERS, FileOpenDialog, FileSaveDialog, IFileDialog,
            IFileOpenDialog, IFileSaveDialog, IShellItem, SHCreateItemFromParsingName, SIGDN_FILESYSPATH,
        },
    },
    core::{HSTRING, Interface},
};

unsafe fn start_in(d: &IFileDialog, folder: &str) {
    if folder.is_empty() {
        return;
    }
    if let Ok(item) = unsafe { SHCreateItemFromParsingName::<_, _, IShellItem>(&HSTRING::from(folder), None) } {
        let _ = unsafe { d.SetFolder(&item) };
    }
}

unsafe fn result(d: &IFileDialog) -> Option<String> {
    let item = unsafe { d.GetResult() }.ok()?;
    let p = unsafe { item.GetDisplayName(SIGDN_FILESYSPATH) }.ok()?;
    let s = unsafe { p.to_string() }.ok();
    unsafe { CoTaskMemFree(Some(p.0 as _)) };
    s
}

/// Ask for a folder, starting in `start`. None if cancelled.
pub fn pick_folder(owner: isize, start: &str) -> Option<String> {
    unsafe {
        let d: IFileOpenDialog = CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).ok()?;
        d.SetOptions(d.GetOptions().ok()? | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM).ok()?;
        let fd: IFileDialog = d.cast().ok()?;
        start_in(&fd, start);
        fd.Show(Some(HWND(owner as _))).ok()?;
        result(&fd)
    }
}

/// "Save as" for a download, suggesting `path`. None if cancelled.
pub fn save_file(owner: isize, path: &str) -> Option<String> {
    let p = std::path::Path::new(path);
    let dir = p.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
    let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    unsafe {
        let d: IFileSaveDialog = CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER).ok()?;
        d.SetOptions(d.GetOptions().ok()? | FOS_OVERWRITEPROMPT | FOS_FORCEFILESYSTEM).ok()?;
        let fd: IFileDialog = d.cast().ok()?;
        start_in(&fd, &dir);
        let _ = fd.SetFileName(&HSTRING::from(name));
        fd.Show(Some(HWND(owner as _))).ok()?;
        result(&fd)
    }
}

/// `dir\name`, or `dir\name (1).ext` and so on if that file already exists.
pub fn unique_path(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let p = std::path::Path::new(name);
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| name.to_string());
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    (1..1000).map(|i| dir.join(format!("{stem} ({i}){ext}"))).find(|c| !c.exists()).unwrap_or(first)
}
