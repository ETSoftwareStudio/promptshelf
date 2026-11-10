---
id: welcome
title: Welcome to PromptShelf
kind: template
tags: [start-here]
vars:
  name:  { hint: "your first name", default: "friend" }
  task:  { from: conversation, hint: "something you ask an AI often" }
  tone:  { choices: [short and direct, friendly, step by step], default: short and direct }
  clip:  { from: clipboard, hint: "copy some text first, then open this prompt" }
---
Hi {{name}}, this is your first prompt. You can delete it when you are done.

A prompt is a Markdown file in your library folder. Your name above was a blank: a place you fill in. In the file, a blank is a word in double braces. Fill the blanks, then copy or paste the result into any AI app.

Try the blanks in this prompt:
- Type your name. It stays filled the next time.
- Pick a tone: {{tone}}.
- Write a task you ask an AI often: {{task}}.
- This blank comes from your clipboard: {{clip}}

Now use the filled prompt. Paste this into an AI app:

"Help me turn this into a reusable prompt with blanks: {{task}}. Keep it {{tone}}."

What to do next:
- Make your own prompt with New prompt (⌘N).
- Pin the prompts you use most. Pins come first, and ⌘1 to ⌘9 open them.
- Edit any prompt as plain Markdown. Your files stay yours, in a folder you chose.
- Connect your library to a private GitHub repo in Settings, to sync your Macs.
- Add a team library in Settings to share prompts with your team.
- In Settings, under Agents, let Claude Code and other agents use your prompts.

When you no longer need this prompt, delete it. Nothing else depends on it.
