# Parallax Scroller Lab

Crate-local verification app for [`saddle-rendering-parallax-scroller`](../..). It keeps the shared crate runnable, BRP-friendly, and E2E-testable without relying on project-level sandboxes.

## Run

```bash
cargo run -p saddle-rendering-parallax-scroller-lab
```

## Run E2E scenarios

```bash
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_scroller_smoke
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_camera_motion
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_finite_bounds
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_zoom
cargo run -p saddle-rendering-parallax-scroller-lab --features e2e -- parallax_pixel_snap
```

For batch runs outside E2E:

```bash
PARALLAX_SCROLLER_EXIT_AFTER_SECONDS=3 cargo run -p saddle-rendering-parallax-scroller-lab
```

## BRP / live inspection

```bash
cargo run -p saddle-rendering-parallax-scroller-lab
BRP_PORT=15742 uv run --project .codex/skills/bevy-brp/script brp resource get saddle_rendering_parallax_scroller::resources::ParallaxDiagnostics
BRP_PORT=15742 uv run --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
BRP_PORT=15742 uv run --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/saddle_rendering_parallax_scroller_lab.png
```

The lab scene includes:

- a forest-style camera-follow stack
- a finite vista strip with horizontal clamping
- a pixel-snap comparison pair
- an auto-scrolling starfield overlay
- runtime overlay text backed by `ParallaxDiagnostics`
