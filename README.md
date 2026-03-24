# Linux GPU Monitor

An OpenAction ([OpenDeck](https://github.com/nekename/OpenDeck)) plugin for displaying GPU statistics on your Stream Deck.

Supports NVIDIA GPUs (via `nvidia-smi`) and AMD GPUs (via `sysfs`).

## Actions

- GPU Utilization
- GPU Temperature
- GPU Memory (VRAM)
- GPU Power

## Building

Requires [Rust](https://www.rust-lang.org/tools/install).

```bash
# Build and create linux-gpu-monitor.zip
./build.sh

# Build and install to local OpenDeck plugins directory
./build.sh --install
```

After installing, reload the plugin in OpenDeck to apply changes.

## Releasing

Pushing a version tag triggers the CI to build for both x86_64 and aarch64, package the plugin, and create a GitHub release with the `.zip` attached.

```bash
git tag v1.0.0
git push origin v1.0.0
```
