---
id: strict-pr-review
title: Strict PR review
kind: template
tags: [review, code]
vars:
  focus:    { from: conversation, hint: "what the user is worried about" }
  findings: { choices: [Blockers only, Everything], default: Blockers only }
  diff:     { from: "git diff --staged" }
---
Review this diff like a strict senior reviewer. Focus on {{focus}}.
Report {{findings}} and skip style nits.

{{> house-rules}}

The diff:
{{diff}}
