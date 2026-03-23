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
