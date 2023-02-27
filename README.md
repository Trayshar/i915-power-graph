# i915-power-graph
Small linux CLI tool to output power usage of Intel Graphics
## Motivation
Right now, the i915 graphics driver is used for many Intel GPUs. Sadly, [hardware monitoring isn't yet fully implemented](https://github.com/torvalds/linux/blob/master/drivers/gpu/drm/i915/i915_hwmon.c), leaving out important data like power usage.

This tool reads the "accumulated energy" sensor repeatedly and artificially calculates the current power usage. This data is presented in a graph using [tui-rs](https://github.com/fdehau/tui-rs).

Tested to work with my Intel Arc A770. Should work for integrated graphics as well.