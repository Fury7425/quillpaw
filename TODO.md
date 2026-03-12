# Tauri Build Fix - Remove problematic dialog plugin, switch to core async dialog

## Steps
- [x] Step 1: Update src-tauri/Cargo.toml - Remove tauri-plugin-dialog dep, add "dialog-open" feature to tauri
- [x] Step 2: Update src-tauri/src/lib.rs - Remove .plugin(tauri_plugin_dialog::init())
- [x] Step 3: Update src-tauri/src/commands/fs_commands.rs - Remove plugin use/import, replace spawn_blocking folder picker with app.dialog().directory().pick_folder().await
- [ ] Step 4: Verify Rust deps with `cd src-tauri && cargo check`
- [ ] Step 5: Test full build with `npm run build`
- [ ] Step 6: Test tauri build `cd src-tauri && cargo tauri build`

