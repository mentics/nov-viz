# Visualization stack decisions

**Date:** 2026-09-02
**Status:** Adopted

## Chosen stack

| Layer | Crate | Role |
|-------|-------|------|
| Graph truth | **petgraph** | Store nodes, typed edges, tags; diff-friendly |
| Layout | **elkrs** | Compute positions and edge routes (projections) |
| Canvas | **gpui-flow** | Editable GPUI graph view (pan, zoom, edit, undo) |

## Why editable graph matters

Nov graphs are not read-only. An agent may propose a layout; the user corrects nodes and edges inline. Those edits become a **delta** we can send back to the agent. Editing, undo, and drag are first-class requirements — not optional polish.

## petgraph

- De facto Rust graph data structure; aligns with future static-analysis work in tod.
- Stable indices, serde support, DOT export for debugging.
- Holds nov primitives (labels, tags, edge types) as node/edge weights.
- Layout and rendering are **projections** — petgraph stays the source of truth.

**Rejected alternatives:** ad-hoc `HashMap` graphs (no algorithm ecosystem), storing layout only in the UI (loses diff/replay).

## elkrs (Eclipse Layout Kernel)

Force-directed layouts (fdg, fa2) alone do not meet nov needs: alignment, orthogonal edge routing, compound nodes, and constraint-aware placement.

ELK provides multiple algorithms behind one API:

- **layered** — dependency flow, orthogonal routing
- **stress** — stable concept-map overview
- **disco** — disconnected clusters (nov “constellations”)
- **interactive** — preserve user-pinned positions

Property/tag-driven layout is achieved by pre-processing into ELK options (clusters, priorities, fixed positions), not by a single “layout by tag” API.

**Rejected alternatives:**

| Option | Reason |
|--------|--------|
| fdg / fa2 only | Force-only; no alignment or routing |
| dagre alone | Good for DAGs; less capable than ELK for compound/routing |
| Graphviz (vizoxide) | Heavy C dependency; weaker interactive story |
| gpui-node-graph | Visual-programming editor (ports/wires); ~2 weeks old; wrong abstraction |

## gpui-flow

- GPUI-native — same stack as tod (`gpui` + optional `gpui-component`).
- React Flow–style: draggable nodes, connect/edit/delete, undo, minimap, custom node renderers (nov cards/atoms).
- **Not on crates.io** — git dependency from [pacifio/gpui-flow](https://github.com/pacifio/gpui-flow).

**Rejected alternatives:**

| Option | Reason |
|--------|--------|
| egui_graphs | egui, not GPUI — separate UI stack |
| Custom GPUI canvas only | Reinvents pan/zoom/edit/undo |
| gpui-node-graph | Node-editor ports/wires; too new and wrong model |
| Web embed (Cytoscape, etc.) | Webview complexity in a native app |

## Architecture principle

```
petgraph (truth) → elkrs (layout projection) → gpui-flow (view + edit) → diff back to truth
```

Graph truth never lives in the layout engine or the canvas alone.

## Open risks

1. **gpui-flow is git-only** — pin revision; watch GPUI version alignment with tod.
2. **elkrs is young** (0.1.x) — validate layout output on fixture graphs early.
3. **Version skew** — gpui-flow may track zed `gpui` git; demo crate may diverge from tod’s crates.io `gpui` until unified.
