# Profiles

A **profile** is a concrete application of the nov abstract framework to a specific context. It specifies how visual units, navigation, compression, and salience work for that case.

## What a profile defines

- **Entry** — how the user arrives (resume last position, hot/salient items first, terrain overview)
- **Unit sizing** — default atom/card/pane density; compression behavior
- **Salience** — what "important" means in this context
- **Format rules** — how information is structured visually
- **Interaction** — keyboard/mouse behavior (when applicable)
- **Anti-patterns** — what to avoid

Profiles do not redefine primitives. They configure the framework.

## Example profiles

| Profile | Context | Status |
|---------|---------|--------|
| [Response](profiles/response.md) | AI assistant communicating to user | draft |
| PR Review | Comprehend a code change in seconds | planned |
| Requirements | Specs and documentation statements | planned |
| Dashboard | SRE / monitoring / operational status | planned |
| Exploration | Learning an unfamiliar codebase | planned |

## Adding a profile

1. Create `profiles/<name>.md`
2. Add entry to the table above
3. Define entry, units, salience, format rules, anti-patterns
4. Include at least one example rendered in the profile's format
