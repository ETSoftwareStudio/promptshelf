---
title: Explain like a colleague
kind: template
tags: [writing, explain]
vars:
  topic:    { from: conversation, hint: "what to explain" }
  audience: { default: a developer new to this codebase }
---
Explain {{topic}} to {{audience}}. Start with what it does, then how it works.
Use one short example. {{> about-me}}
