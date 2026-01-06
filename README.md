# smolBSD-tui

smolBSD-tui is a TUI (Text User Interface) to handle [smolBSD](https://github.com/netbsdfr/smolBSD/) VMs

It is written in Rust using the [ratatui](https://ratatui.rs/) library

# Features
- [X] Start a VM
- [X] Stop a VM
- [X] Delete a vm
- [ ] Add scrollbar on popups when needed
    - cf https://docs.rs/ratatui/0.30.0-alpha.5/ratatui/widgets/struct.Scrollbar.html#examples
    - `src/ui/ui.rs`, `get_centered_area_fit_to_content()` and `render_confirmation_popup()`
- [ ] Create a new VM
- [ ] Edit an exiting VM
- [ ] Display the CPU usage
- [ ] Use the `notify` crates to reload the app state when a file changes in `etc/`, `images/` or `kernel/`
- [ ] Connect to the console
- [ ] Filter kernels/images filenames
- [ ] Add smolBSD logo in the top right corner
- [ ] Create binaries for multiple architectures (macos-amd64, macos-aarch64, linux-amd64, linux-aarch64, ???) (using musl? cf https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance/)
