# Promem Roadmap

This roadmap turns the full vision in `docs/vision.md` and the memory-engine
addendum in `docs/vision-p2.md` into versioned, shippable milestones.

Each version should have a clear user-facing outcome, a small implementation
surface, and tests that prove the release is useful on its own. Keep this
roadmap focused on decisions and deliverables; use `docs/vision.md` for the
long-form product rationale.

## Versions

- [`v0.md`](v0.md): local Markdown memory MVP
- [`v1.md`](v1.md): Git-native workflows
- [`v2.md`](v2.md): extraction and provider automation
- [`v3.md`](v3.md): MCP integration
- [`v4.md`](v4.md): packaging and distribution
- [`v5.md`](v5.md): search intelligence and graph features
- [`v6.md`](v6.md): editor and assistant integrations
- [`v7.md`](v7.md): team workflows and external importers

## Memory Engine Addendum

- [`a0.md`](a0.md): smart search foundation
- [`a1.md`](a1.md): smart context loading
- [`a2.md`](a2.md): memory graph
- [`a3.md`](a3.md): memory quality
- [`a4.md`](a4.md): repo importers

## Release Rule

Promem should stay local-first and Git-native across every version:

- durable memory is Markdown committed with the repository
- generated caches stay out of Git
- JSON contracts remain stable for automation
- no raw transcript storage
- integrations call the core engine instead of writing `.promem/` directly
