# Architecture

## Overview

`saddle-rendering-parallax-scroller` is built around two public concepts:

1. **Rigs**: one independent parallax coordinate space, optionally bound to one camera
2. **Layers**: sprites that derive an effective offset from the rig camera state, manual phase, and auto-scroll

There is no global singleton. Multiple rigs can coexist in one world, share the same camera, or bind to different cameras.

## Runtime Flow

The plugin uses five public phases:

1. `TrackCamera`
2. `UpdateOffsets`
3. `ApplyLayout`
4. `Diagnostics`
5. `Debug`

The default chain is:

```text
TrackCamera -> UpdateOffsets -> ApplyLayout -> Diagnostics -> Debug
```

### `TrackCamera`

- ensure internal rig state exists
- resolve the bound camera entity, if any
- read current camera position
- compute the current viewport size in world units from `Camera::logical_viewport_rect()` plus `viewport_to_world_2d`
- if no camera is bound, use zero camera input instead of reusing the rig's world transform

Using the camera viewport conversion path instead of hand-rolled projection math keeps the crate resilient to:

- orthographic zoom changes
- camera viewport splits
- logical vs physical window scaling

### `UpdateOffsets`

- ensure internal layer state exists
- accumulate `auto_scroll * dt`
- skip accumulation for disabled layers
- freeze accumulation for all layers under a disabled rig

The accumulated auto-scroll phase is stored once per layer. No child churn happens here.

### `ApplyLayout`

For each layer:

1. resolve one-segment source size
2. compute the unbounded offset
3. wrap or clamp it per axis
4. snap it if requested
5. write the final layer transform
6. apply the selected render strategy

Disabled rigs freeze their previously resolved layout. This keeps runtime toggles honest: disabling a rig stops both camera tracking and auto-scroll advancement without despawning the layer stack.

## Offset Math

The core local offset is:

```text
camera_position * camera_factor
+ manual_phase
+ accumulated_auto_scroll
```

This value is then transformed per axis:

- repeat enabled: wrap into a centered modulo interval
- repeat disabled with bounds: clamp into the configured range
- repeat disabled without bounds: leave unchanged

Finally the snap mode is applied:

- `None`: no post-process
- `Pixel`: round to whole units
- `Grid(step)`: round to the provided step per axis

The final layer translation is:

```text
rig.origin + layer.origin + snapped_offset
```

When the rig entity itself also has a transform, Bevy's normal parent-child transform propagation applies on top.

## Strategy Choice

### Tiled Sprite Strategy

`TiledSprite` uses Bevy's native `SpriteImageMode::Tiled`.

Why this exists:

- ideal for seamless textures
- zero clone management
- clean support for two-axis repeat

How coverage is chosen:

- start from current viewport size
- add `coverage_margin` on repeating axes
- clamp upward to `minimum_coverage`
- never go below the source tile size

The tile span itself is derived from:

```text
source_size * abs(scale) * stretch_value
```

### Segmented Strategy

`Segmented` uses the layer entity as the center segment and manages a stable grid of child clones around it.

Why this exists:

- authored strips are often not meaningfully tileable through UV repetition
- repeated sprite clones are easier to reason about for silhouettes and card-like scenic layers

How segment coverage works:

- compute one segment's world size from `source_size * abs(scale)`
- compute the visible half-count needed for the viewport
- add `extra_rings`
- keep the total count odd so the layer entity stays the center segment

There is no per-frame respawn churn. Children are only spawned or despawned when the required grid shape changes.

## Zoom Awareness

Viewport size is recomputed from the bound camera every frame. That feeds both:

- tiled coverage size
- segmented child-grid count

This is the mechanism that prevents zooming out from exposing gaps.

## Debugging Surface

`ParallaxDiagnostics` mirrors the live runtime state in a BRP-friendly resource:

- per rig: camera target, camera position, viewport size
- per layer: strategy, effective offset, wrap span, coverage size, segment grid

`ParallaxDebugSettings` enables optional gizmo drawing for:

- current viewport bounds
- layer coverage bounds
- wrap spans
- offset vectors

## Key Invariants

- every layer belongs to one rig via `ChildOf`
- camera tracking is optional, but viewport-aware coverage is only available when a camera is bound
- unbound rigs use zero camera motion rather than double-applying the rig transform
- layer motion is fully derived from components plus time, not from transient spawn/despawn effects
- repeat math is symmetric for positive and negative travel
- segment grids stay centered and odd-sized
