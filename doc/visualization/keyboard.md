# Nov graph keyboard

**Status:** Adopted for `nov-viz` demo
**Defaults:** remappable (`Keymap`); this file is the meaning of the verbs, not a frozen physical layout.

Two independent axes. Selection never drives the camera except for a courtesy keep-in-view after Move. Camera never changes selection.

| | Selection | Camera |
|--|-----------|--------|
| Translate | **Move** | **Pan** |
| Depth / scale | **In** / **Out** | **Zoom in** / **Zoom out** |

## Verbs

| Verb | Axis | Effect |
|------|------|--------|
| Move | Selection | Move the **cursor** to a spatial neighbor |
| In | Selection | One level deeper |
| Out | Selection | One level shallower |
| Pan | Camera | Slide the viewport; selection and cursor unchanged |
| Zoom in / out | Camera | Scale the viewport; selection and cursor unchanged |
| Fit view | Camera | Frame all nodes (`G`) |
| Edit | Content | Start mutating the current stop’s content |
| Exit edit | Content | Leave edit mode; **keep the buffer** (no revert) |
| Create | Structure | New peer at the current depth, near the cursor |
| Delete | Structure | Remove the selection (or the cursor node if the selection is empty) |
| Connect | Structure | Edge(s) from the selection to a target |
| Toggle select | Selection | Add/remove the cursor node without moving |

**Escape is not Out.** It is context-specific (exit edit, cancel connect). Whether Escape should also Out at the graph surface is deferred — try both in use; default is **no**.

## Cursor vs selection

- **Cursor** — keyboard focus. Exactly one node, or none. After Move, the viewport **soft-pans** so the cursor stays on screen.
- **Selection** — set of nodes for delete / connect / batch ops.
- Without modifiers, Move sets `selection = {cursor}`.
- **Shift+arrows** — move cursor and **extend spatially**: select every node whose center lies in the bounding box of the shift-anchor and the new cursor.
- **Ctrl+Space** — toggle the cursor node in/out of the selection; cursor does not move. Lets you pick nodes one at a time, then **C**.

Shift+WASD is **Pan**, not extend. Arrow keys keep the familiar select-and-shift-extend behavior; WASD stays a left-hand cluster (move vs camera via Shift).

## Depth stack (selection)

```text
Graph surface → Node interior → Edit session
```

| Here | In (`R` / Enter) | Out (`E`) | Edit (`Q`) |
|------|------------------|-----------|------------|
| Graph, cursor on a node | Enter that node | Clear selection; cursor stays | Edit label (skip interior) |
| Graph, no cursor | No-op | No-op | No-op |
| Node interior | Enter edit on the highlighted stop | Back to graph; cursor stays | Edit current stop |
| Edit session | *(not a nav key — see edit)* | Types `e` if you press E | — |

v0 cards have one stop (the label). Interior still exists so In/Out stay learnable when atoms land later.

## Edit session (same as tod)

| | Single-line (node labels) | Multi-line (later) |
|--|---------------------------|-------------------|
| **Enter** | Exit edit (soft-done, keep text) | Newline |
| **Ctrl+Enter** | Exit edit | Exit edit (soft-done, keep text) |
| **Escape** | Exit edit, keep text | Exit edit, keep text |

Graph nav (Move/In/Out/Create/…) does not fire while editing. Camera Zoom/Pan **may** still run so the viewport is not trapped.

## Create / connect / empty graph

- **F** — new node. Desired point is the **cursor** (node center, or viewport center if none). Placement is the nearest slot the layout rules allow that does not overlap existing nodes (v0: grid/spiral search; later: ELK interactive).
- **C** — start connect from the **selection** (if empty, from the cursor). Move the cursor to the target; **R**/Enter commits edges from every source to the target (skip self / duplicates). **E** or **Escape** cancels.
- Multi-select then **C** then commit → many-to-one (or chain later). Fine for “pick several, connect them.”
- Empty graph: In is a no-op; **F** still creates; Pan/Zoom/Fit always work.

## Default keys (QWERTY, remappable)

Left-hand cluster:

```text
Q  W  E  R     Q=Edit   W=↑     E=Out    R=In
A  S  D  F     A=←      S=↓     D=→      F=New
   X           X=Delete
         G     G=Fit view
         C     C=Connect
```

| Verb | Primary | Also |
|------|---------|------|
| Move | `W A S D` | `↑ ↓ ← →` |
| In | `R` | `Enter` |
| Out | `E` | — |
| Edit | `Q` | — |
| Create | `F` | — |
| Delete | `X` | `Backspace`, `Delete` |
| Connect | `C` | — |
| Toggle select | `Ctrl+Space` | — |
| Extend select | `Shift+arrows` | — |
| Pan | `Shift+W A S D` | held continuously |
| Zoom in | `Shift+R` | held continuously |
| Zoom out | `Shift+E` | held continuously |
| Fit view | `G` | — |
| Undo / Redo | `Cmd/Ctrl+Z` | `Cmd/Ctrl+Shift+Z` |
| Exit edit | `Escape` | Enter / Ctrl+Enter as above |

**Camera while held is time-based**, not OS key-repeat. Selection Move may use normal key repeat.

## Cheatsheet

**Selection:** WASD/arrows move · R/Enter in · E out · Q edit · F new · X delete · C connect · Ctrl+Space toggle · Shift+arrows extend
**Camera:** Shift+WASD pan · Shift+R / Shift+E zoom · G fit
**Edit:** Escape/Enter leave (keep text) · Ctrl+Enter leave on multi-line

## Tod

Graph ships this map first. Forms keep today’s Enter-to-edit / Escape-to-leave. Align `Q` / WASD onto tod later; shared rule: **Escape always leaves the edit session before anything else.**
