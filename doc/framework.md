# Abstract Framework

## Core thesis

Information is a **graph**. Display is a **projection** of that graph into a view the user can navigate — like a transformation from an abstract structure into something spatial and glanceable.

The user does not read; they **scan terrain, land on atoms, and jump**.

## Primitives

### Node

Anything that can hold information and be navigated into. Nodes are not limited to one visual form — an atom is a node, a card is a node, a concept in a graph is a node.

### Edge

A typed relationship between two nodes.

- **Specific:** `depends on`, `blocks`, `replaces`, `causes`, `affects`
- **Generic:** `related` — valid when the relationship is known but not yet named; refine later

Edges can be refined over time. The graph is living, not finalized.

### Label

A short visual descriptor on a node — answers "what is this particular thing?"

- Describes the **instance**, not the category
- Optimized for **decision**: does this match what I'm looking for?
- Glanceable: roughly one fixation (size adjustable per user)

Examples: `auth: broken`, `reconnect.rs`, `JWT replaces sessions`

### Tag

A categorization marker on a node — answers "what class does this belong to?"

- Describes a **category**, not the instance
- Multiple tags per node; tags create implicit edges between nodes sharing a tag
- Used for filtering and faceting

Examples: `security`, `breaking`, `rust`, `infra`

**Label vs tag:** label = this thing; tag = what kind of thing.

## Visual units

Display size hierarchy (all adjustable per user):

| Unit | Role |
|------|------|
| **Atom** | Smallest comprehension unit. One glance, one fact. |
| **Card** | Cluster of related atoms. One concept. |
| **Pane** | Everything visible without scrolling. The working field. |

These are visual components, not data types. A card *is* a node; it *contains* atom nodes.

## Graph structure

- Truth lives in the **graph** — nodes and typed edges
- Not necessarily one monolithic graph; may be **constellations** of clusters with bridges between them
- A node can participate in multiple clusters (multiple parents, cross-links)
- Trees and taxonomies are **projections**, not requirements — useful sometimes, never forced

## Projection

The graph does not inherently live in 2D space. **Layout is a view** — a projection of a graph subset into a spatial arrangement.

- Multiple valid projections of the same graph
- A node may appear in more than one place if it bridges clusters
- Projection can be user-defined, learned (desire paths), or algorithmically suggested
- Spatial proximity can encode similarity, but is never the sole truth

Think in terms of **transformations**: graph → projection → pane.

## Depth (semantic zoom)

Drilling into a node reveals more detail at increasing resolution. Not limited to a fixed number of levels.

```
L0  label glance          "auth"
L1  status                "auth: broken on reconnect"
L2  causal fact           "race in socket handshake"
L3  affected neighbors    "→ deps, → reconnect.rs"
L4  code / evidence       (lines, stack trace, diff hunk)
L5+ further detail        (history, related changes, tests)
```

When drilling in (moving along depth), the projection (where you are in the graph) must remain perceptible — you never lose your place.

## Path

The sequence of nodes visited during exploration. Path is **context**:

- Informs salience of unvisited nodes (connected to what you've seen → more interesting)
- Enables forward/back navigation through rabbit holes
- Compresses display (atoms at depth N can assume path context; no re-explaining)
- Can be used as a **query** into the unvisited graph

Path is branching, not linear. Research basis: hoptrees, PadPrints — branching history outperforms breadcrumbs for non-linear exploration.

## Salience

What matters *right now* — context-dependent, path-informed.

- Highlights and suggests; **never reorders stable slots** (muscle memory)
- Domain-specific: "broken" matters on an SRE dashboard; "prerequisite" matters in a math explanation
- Path proximity boosts salience: nodes connecting to recent visits rank higher

## Stable slots and desire paths

**Tension:** relevance wants to reorder; muscle memory requires stability.

**Resolution:**

| Stable (never moves) | Dynamic (changes freely) |
|----------------------|--------------------------|
| Slot positions in pane | Highlight on salient items |
| Keybinding → action mapping | Suggested next hops |
| User-pinned routes | Path-derived interest scores |
| User-defined projections | Atom density / compression |

**Desire paths:** frequently traveled routes get promoted to first-class shortcuts (the university sidewalk pattern — observe where people walk, then pave it). User defines or accepts suggested routes; system reinforces them.

## Dynamic unit sizing and compression

Display adapts to the user:

| Factor | Effect |
|--------|--------|
| Fatigue | Shrinks all unit levels |
| Domain expertise | Familiar concepts collapse into larger units |
| Notation fluency | Dense symbols become single atoms (expert math notation) |
| Path familiarity | Traversed chains compress ("auth→token→refresh" → one atom) |

Single-user focus first. Per-user profiles when multiple users share an application.

**Foveal load:** unfamiliar material forces character-by-character parsing; familiar material compresses into chunks processable in one glance. The system should recognize and reflect this.

## Saccade-friendly design

- Atoms are **landing targets**, not words in a sentence
- Spatial arrangement matters more than reading order
- Long prose forces small sequential saccades — avoid it
- Large exploratory saccades (scanning, jumping) are the natural mode; design for them

## Perceptual span (starting guidance)

Research basis: ~14–15 characters per fixation for skilled English readers; span shrinks under load.

- Atom: one fixation, adjustable (user may handle more when fresh/expert)
- Card: cluster of atoms
- Pane: full visible field (more than a handful of cards)

All levels adjust together under dynamic compression.

## Rules (summary)

1. Atoms, not prose
2. Glanceable units (adjustable)
3. Saccade-friendly spatial layout
4. Graph is truth; projection is view
5. Depth without losing place
6. Path is context and query
7. Salience highlights; slots stay stable
8. Desire paths become shortcuts
9. Edges refine over time (`related` is fine)
10. Expertise compresses familiar material
