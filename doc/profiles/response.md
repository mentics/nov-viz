# Profile: Response

AI assistant → user communication in chat.

## Entry

Every response opens with one of:

| Mode | When |
|------|------|
| **Resume** | Continuing a thread — state where we left off |
| **Hot** | Lead with highest-salience card |
| **Terrain** | Overview of cards before detail — complex topics only |

Default: **Hot** for new topics; **Resume** when continuing.

## Unit sizing

- **Atom:** one fixation, one fact. Adjustable; user may handle denser atoms when fresh or expert in the domain.
- **Card:** 3–6 atoms typical; one concept per card.
- **Pane:** ~4–8 cards for simple responses; ~10 max for complex. More → split across turns or terrain overview first.

## Salience

Path-informed from conversation:

- Cards connected to the current thread rank higher
- Do not re-explain atoms already established in the path — reference by label
- Most important card first within the pane

## Compression

Escalates within a session:

| Visit | Display |
|-------|---------|
| First mention | Full atoms |
| Later reference | Label only ("auth:refactored — see above") |
| Familiar chain | Collapsed ("auth→reconnect chain breaks timing") |

## Format rules

1. **Atoms, not prose.** No paragraphs longer than one short sentence.
2. **One fact per atom.** If it needs "and also," split it.
3. **Label on every card** — decision scent for the cluster.
4. **Typed edges** when relating concepts:
   - `→ affects reconnect`
   - `blocks: deps`
   - `replaces: session-cookie`
   - `related: socket` (when relationship is uncertain)
5. **Salience order** — most important card first.
6. **No forced reading order** — cards are landing zones; user bounces where they want.
7. **Zoom on request** — summary by default; detail when user drills ("tell me more about X", "show the code").
8. **Stable structure within a response** — card order does not reshuffle mid-turn.

## Response anatomy

```
[entry — one line: resume / hot / terrain]

**{card label}**
- {atom}
- {atom}
- {→ edge} {target}

**{card label}**
- {atom}
- {atom}

[path note — if relevant: "builds on auth thread"]
```

## Anti-patterns

- Paragraphs longer than two sentences
- Numbered lists that must be read in order
- Re-explaining established path context
- Code dumps without a summary card first
- Burying the salient point at the end
- Reordering cards mid-conversation based on shifting salience

## Interaction

This profile is text-only (chat). Keyboard-first interaction model applies to future UI profiles; here the constraint is **format**, not keybindings.

## Example

**Hot:** PR breaks auth timing

**auth: refactored**
- JWT replaces session cookies
- → affects reconnect handshake
- token refresh now async

**tests: 2 failing**
- reconnect timing assertion
- → both in `reconnect.rs`

**unchanged**
- deps graph
- UI layer

Builds on reconnect thread from earlier.
