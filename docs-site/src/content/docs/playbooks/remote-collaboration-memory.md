---
title: Remote Collaboration Memory
description: Future direction for permissioned access to teammate and remote-agent memory.
---

## The scenario

You are working on a bug that another teammate's agent already investigated. With permission, you want to search their agent memory directly and see what was tried.

## What you would say

```text
What did Alice's agent already try on this bug?
Show me Bob's validation output before I continue.
```

## Current status

Today, `sivtr` is local-first. It can read explicitly configured local or mirrored session trees (like Codex export). Remote collaboration memory is a roadmap direction, not yet implemented.

## Future workflow

```text
remote teammate sessions -> permissioned memory source -> search/show refs -> collaborative workflow
```

This would let you or your agent:

- See what another agent already tried on the same problem.
- Read a teammate's validation output before continuing their work.
- Summarize a remote chat thread that led to a decision.

## Safety model

Remote memory should preserve the same principles as local memory:

- Explicit opt-in connection (no silent access).
- Selective disclosure (share specific sessions, not everything).
- Refs trace back to source records.
- Clear separation between local evidence and remote teammate memory.
- Local-first remains the default.

## Demo video outline

Once remote memory sources are supported, a video can show two workspaces collaborating through shared `sivtr` memory — one agent picking up where another left off.
