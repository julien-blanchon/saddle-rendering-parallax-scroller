# Architecture

## Overview

`saddle-rendering-parallax-scroller` is built around two public concepts:

1. **Rigs**: one independent parallax coordinate space, optionally bound to one camera
2. **Layers**: sprites that derive an effective offset from the rig camera state, manual phase, and auto-scroll

There is no global singleton. Multiple rigs can coexist in one world, share the same camera, or bind to different cameras.

## Runtime Flow

The plugin uses six public phases:

1. `TrackCamera`
2. `UpdateOffsets`
3. `ComputeOffsets`
4. `WriteTransforms`
5. `Diagnostics`
6. `Debug`

The default chain is:

```text
TrackCamera -> UpdateOffsets -> ComputeOffsets -> [user hook] -> WriteTransforms -> Diagnostics -> Debug
```

### `TrackCamera`

- ensure internal rig state exists (`RigRuntimeState`)
- resolve the bound camera entity, if any
- read current camera position
- compute the current viewport size in world units from `Camera::logical_viewport_rect()` plus `viewport_to_world_2d`
- if no camera is bound, use zero camera input instead of reusing the rig's world transform

Using the camera viewport conversion path instead of hand-rolled projection math keeps the crate resilient to:

- orthographic zoom changes
- camera viewport splits
- logical vs physical window scaling

### `UpdateOffsets`

- ensure internal layer state exists (`LayerRuntimeState`, `ParallaxLayerComputed`)
- accumulate `auto_scroll * dt * time_scale * rig.speed_multiplier`
- skip accumulation for disabled layers
- freeze accumulation for all layers under a disabled rig

The accumulated auto-scroll phase is stored once per layer. No child churn happens here.

### `ComputeOffsets`

For each layer:

1. resolve one-segment source size
2. compute depth ratio (for perspective cameras with depth mapping)
3. derive effective camera factor and scale
4. compute the unbounded offset: `camera_pos * factor + phase + auto_phase`
5. wrap or clamp per axis
6. snap if requested
7. write the result to `ParallaxLayerComputed { offset, scale, depth }`
8. fire `ParallaxLayerWrapped` messages when wrapping occurs

This phase does **not** touch `Transform` or `Sprite`. It only writes to `ParallaxLayerComputed` and `LayerRuntimeState`.

### Custom Offset Hook

Between `ComputeOffsets` and `WriteTransforms`, users can schedule their own systems to read and modify `ParallaxLayerComputed`:

```rust
app.add_systems(
    Update,
    my_wobble_system
        .after(ParallaxScrollerSystems::ComputeOffsets)
        .before(ParallaxScrollerSystems::WriteTransforms),
);

fn my_wobble_system(time: Res<Time>, mut layers: Query<&mut ParallaxLayerComputed>) {
    for mut computed in &mut layers {
        computed.offset.y += (time.elapsed_secs() * 3.0).sin() * 8.0;
    }
}
```

This is the primary extensibility mechanism. Any offset modification — wobble, shake, event-driven bursts, procedural drift — can be injected here without fighting the crate's transform ownership.

### `WriteTransforms`

For each layer:

1. read `ParallaxLayerComputed` (potentially modified by user systems)
2. add `user_offset` and multiply by `user_scale`
3. write the final `Transform` (translation, scale, rotation)
4. write `Sprite` (color, image_mode, custom_size)
5. manage segmented child entities via `sync_segment_children`

The final layer translation is:

```text
rig.origin + layer.origin + computed.offset + layer.user_offset
```

The final scale is:

```text
computed.scale * layer.user_scale
```

Disabled rigs freeze their previously resolved layout. This keeps runtime toggles honest: disabling a rig stops both camera tracking and auto-scroll advancement without despawning the layer stack.

## Time Scale and Speed Control

Two mechanisms control auto-scroll speed:

- **`ParallaxTimeScale`** (global resource): multiplies `dt` for all rigs. `0.0` = paused, `1.0` = normal, `2.0` = double speed.
- **`ParallaxRig::speed_multiplier`** (per-rig): multiplies `dt` for that rig's child layers only.

The effective dt for auto-scroll is: `time.delta_secs() * time_scale.0 * rig.speed_multiplier`

Neither affects camera-factor (spatial parallax ratio) — only auto-scroll accumulation.

## Public Runtime State

Both `RigRuntimeState` and `LayerRuntimeState` are public components, queryable from user systems:

- **`RigRuntimeState`**: camera position, camera depth, viewport size, perspective flag
- **`LayerRuntimeState`**: effective camera factor, effective scale, auto phase, depth ratio, effective offset, wrap span, coverage size, segment grid

This enables user systems to build on top of the crate — e.g., "spawn particles where the ground layer is" or "scale UI to match parallax viewport".

## Messages

The crate emits buffered messages (Bevy `Message` / `MessageWriter` / `MessageReader`):

| Message | When |
|---------|------|
| `ParallaxActivated` | Runtime activated via activate schedule |
| `ParallaxDeactivated` | Runtime deactivated via deactivate schedule |
| `ParallaxLayerWrapped` | A layer's offset wraps around on a repeating axis |
| `ParallaxSegmentSpawned` | A segment child is spawned for a segmented layer |
| `ParallaxSegmentDespawned` | A segment child is despawned |

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

When the rig entity itself also has a transform, Bevy's normal parent-child transform propagation applies on top.

## Perspective Depth Mapping

`ParallaxDepthMapping` adds an optional physically motivated layer on top of the authored `camera_factor` path.

When all of the following are true:

- the rig is bound to a camera
- that camera uses `Projection::Perspective`
- the layer has `depth_mapping = Some(...)`

the crate computes a depth ratio:

```text
abs(camera_z - reference_plane_z) / abs(camera_z - layer_plane_z)
```

where `layer_plane_z` comes from the rig's world Z plus `ParallaxLayer.depth`.

That ratio drives two derived values:

- `effective_camera_factor = camera_factor + (1 - depth_ratio) * translation_response`
- `effective_scale = scale * lerp(1.0, depth_ratio, scale_response)`

Practical implications:

- background planes (`layer_plane_z` farther than the reference plane) get factors between `0` and `1` and shrink slightly during a dolly-in
- foreground planes can produce negative physical factors, which makes them sweep faster across the frame during camera travel
- orthographic examples continue to behave exactly as before because the mapping simply stays inactive

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
- per layer: strategy, effective camera factor, effective scale, optional depth ratio, effective offset, wrap span, coverage size, segment grid

`ParallaxDebugSettings` enables optional gizmo drawing for:

- current viewport bounds
- layer coverage bounds
- wrap spans
- offset vectors

## Example Texture Architecture

All examples use procedurally generated textures (no external asset files). The `common/` crate provides two tiers:

1. **Legacy textures** (`sky`, `mountains`, `canopy`, `stars`, `pixel_clouds`, `vista`) — simple gradient/pattern textures for the original examples
2. **Rich scene textures** (`forest_*`, `city_*`) — atmospheric multi-layer scenes using fractal noise, value noise with smoothstep interpolation, and layered silhouette generation

The rich textures apply **atmospheric perspective**: far layers use lighter, bluer tones while near layers use darker, more saturated colors. This mimics the effect of light scattering through air, creating a convincing depth illusion even with procedural art.

Texture generators include:
- `forest_sky_gradient` — sky gradient with fbm-based cloud wisps
- `forest_mountain_silhouette` — parameterized mountain ridges from sine wave sums + noise
- `tree_silhouette_strip` — procedural conifer forests with random tree placement and height
- `building_silhouette_strip` — city buildings with optional lit window patterns
- `ground_strip` — noisy textured ground

## Key Invariants

- every layer belongs to one rig via `ChildOf`
- camera tracking is optional, but viewport-aware coverage is only available when a camera is bound
- unbound rigs use zero camera motion rather than double-applying the rig transform
- layer motion is fully derived from components plus time, not from transient spawn/despawn effects
- repeat math is symmetric for positive and negative travel
- segment grids stay centered and odd-sized
- `ParallaxLayerComputed` is always populated before `WriteTransforms` runs
- `user_offset` and `user_scale` are purely additive / multiplicative and never affect the computed pipeline
