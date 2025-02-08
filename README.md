# smolBSD-tui

smolBSD-tui is a TUI (Text User Interface) to handle [smolBSD](https://github.com/netbsdfr/smolBSD/) VMs

It is written in Rust using the [ratatui](https://ratatui.rs/) library

# Missing features
- [X] Start a VM
- [X] Stop a VM
- [X] Refresh vms/kernels/images every 2s
- [X] Delete a vm
- [ ] Create a new VM
- [ ] Edit an exiting VM
- [ ] Display the CPU usage
- [ ] Connect to the console
- [ ] Filter kernels/images filenames
- [ ] Add smolBSD logo in the top right corner
- [ ] Create binaries for multiple architectures (macos-amd64, macos-aarch64, linux-amd64, linux-aarch64, ???) (using musl? cf https://nickb.dev/blog/default-musl-allocator-considered-harmful-to-performance/)
