---
allowed-tools: Bash(git add:*), Bash(git status:*), Bash(git commit:*)
description: Create a git commit
---

## Context

- Current git status: !`git status`
- Current git diff (staged and unstaged changes): !`git diff HEAD`
- Current branch: !`git branch --show-current`
- Recent commits: !`git log --oneline -10`

## Your task

Based on the above changes, create a single git commit.

## Commit Guidelines

- **Explain why, not what**: Commit messages should explain the reason for the change. If you're really not sure why, then prompt the user for clarification.

Example: `feat: add network request filtering to reduce noise in captured data`.
