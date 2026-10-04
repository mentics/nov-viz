# Nov visualization wiring

Concise design for connecting **petgraph → elkrs → gpui-flow**.

## Pipeline

```text
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌──────────────┐
│  NovGraph   │───▶│  ElkLayout   │───▶│  FlowView   │───▶│  User edits  │
│  (petgraph) │    │  (elkrs)     │    │ (gpui-flow) │    │  → diff      │
└─────────────┘    └──────────────┘    └─────────────┘    └──────────────┘
     truth              projection           render            feedback
```

## Crate: `nov-viz`

Standalone workspace member for prototyping. Later: tod depends on `nov-viz` as a library.

```
crates/nov-viz/
  src/
    model.rs      # NovGraph: petgraph + node/edge metadata
    layout.rs     # petgraph → ELK JSON → elkrs → positions
    flow.rs       # positions → FlowNode/FlowEdge; edit callbacks → model
    fixture.rs    # Sample agent-response graph
  examples/
    demo.rs       # Window + fixture; cargo run -p nov-viz --example demo
```

## 1. Model (petgraph)

```rust
// Node weight
struct NovNode {
    id: String,
    label: String,
    tags: Vec<String>,
    width: f64,
    height: f64,
}

// Edge weight
struct NovEdge {
    id: String,
    edge_type: String,  // "depends_on", "blocks", "related", ...
}

type NovGraph = StableDiGraph<NovNode, NovEdge>;
```

- Stable node/edge indices for diffing.
- `width`/`height` feed ELK and gpui-flow node sizing.
- Mutations: `add_node`, `remove_node`, `add_edge`, `remove_edge`, `update_label`.

## 2. Layout (elkrs)

**Input:** `&NovGraph`, `Projection` enum (which layout profile).

**Steps:**

1. Build ELK JSON graph from petgraph (children + edges + layoutOptions).
2. `elkrs::create_elk().layout_json(input)?` → laid-out JSON.
3. Parse x, y, width, height per node; edge bend points if present.

**Default projection for demo:** `Terrain` → algorithm `org.eclipse.elk.stress` or `org.eclipse.elk.disco` for clustered fixture.

**Flow projection:** `Flow` → `org.eclipse.elk.layered` with orthogonal routing for directed edges.

**Pinned / after user edit:** `Interactive` → layered or stress with `nodePlacement.strategy: INTERACTIVE` and existing coordinates.

Mapping sketch (ELK JSON):

```json
{
  "id": "root",
  "layoutOptions": {
    "elk.algorithm": "org.eclipse.elk.stress",
    "elk.spacing.nodeNode": "40"
  },
  "children": [
    { "id": "n1", "width": 140, "height": 56, "labels": [{ "text": "auth: broken" }] }
  ],
  "edges": [
    { "id": "e1", "sources": ["n1"], "targets": ["n2"] }
  ]
}
```

## 3. View (gpui-flow)

**Input:** laid-out nodes (id, label, x, y, w, h) + edges.

**Steps:**

1. Map each node → `FlowNode::new(id, x, y).label(...).size(w, h)`.
2. Map each edge → `FlowEdge::new(id, source, target).label(edge_type)`.
3. `FlowState::new(nodes, edges)` + `FlowGraph::new(state, cx)` with custom `node_renderer` for nov card styling.
4. Register callbacks:
   - **on_connect** / delete / drag end → update `NovGraph`, optionally re-layout or keep positions.
   - **push_undo** on batch edits.

**Re-layout trigger:** explicit user action (“Re-layout”) or projection switch — not on every drag (preserves muscle memory).

## 4. Edit diff (future)

When user edits the canvas, emit:

```rust
enum GraphEdit {
    NodeAdded { id, label, position },
    NodeRemoved { id },
    NodeMoved { id, position },
    LabelChanged { id, label },
    EdgeAdded { id, source, target, edge_type },
    EdgeRemoved { id },
}
```

Agent feedback channel consumes `Vec<GraphEdit>` — not implemented in v0 demo; hooks only.

## Demo example

`examples/demo.rs`:

1. Load `fixture::agent_response_sample()` into `NovGraph`.
2. Run `Terrain` layout via elkrs.
3. Open GPUI window with `FlowGraph` showing the fixture.
4. Title: “Nov Viz Demo — agent response fixture”.

Run: `cargo run -p nov-viz --example demo`

## Dependencies

```toml
petgraph = "0.8"
elkrs = "0.1"
gpui-flow = { git = "https://github.com/pacifio/gpui-flow" }
gpui = "0.2"   # align with tod / gpui-flow
gpui-component = "0.5"   # optional, for themed cards
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
```

Pin `gpui-flow` git rev once verified building.

## Integration path (later)

1. `tod` adds `nov-viz` path dependency.
2. Agent response profile renders into `NovGraph` instead of markdown-only.
3. Panel hosts `FlowGraph` entity; edits flow back to agent session.

## Non-goals (v0)

- Semantic zoom (card → atoms) — separate depth layer
- Path/hoptree UI
- Persistence / serde snapshot

Keyboard verbs and defaults: [keyboard.md](keyboard.md).
