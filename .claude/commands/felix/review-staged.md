# Review Staged Changes

Perform a rapid code review of staged changes, suitable for pre-commit review.

## Usage

```bash
/review-staged
```

## Instructions

You are performing a focused code review of staged changes before commit. This is a single-pass review covering the most important aspects.

### Step 1: Get the Changes

```bash
git diff --cached --stat
git diff --cached
```

### Step 2: Single-Pass Review

Delegate reviews thematically to multiple sub-agents in parallel and have them report back to you.

Review the changes holistically, checking for:

**🚨 Blockers** (must fix before merge)

- Security vulnerabilities (exposed secrets, injection risks)
- Breaking changes without migration path
- Critical bugs or logic errors
- Missing error handling that could cause crashes

**⚠️ Important** (should fix)

- Missing or inadequate tests for new functionality
- Performance concerns (N+1 queries, inefficient loops)
- Poor error messages or logging
- Type safety issues
- Leaky abstractions
- Code smells

**💡 Suggestions** (nice to have)

- Code clarity improvements
- Documentation additions
- Refactoring opportunities
- Better naming

### Step 3: Output Format

```markdown
## Quick Review: [Brief description of changes]

**Verdict**: ✅ Approve | ⚠️ Approve with suggestions | ❌ Request changes

### Summary
[1-2 sentence summary of what the changes do and overall quality]

### Blockers
[List any must-fix issues, or "None"]

### Recommendations  
[List important suggestions]

### Notes
[Any other observations]
```

Keep feedback concise and actionable. For small, clean changes, a brief approval is perfectly appropriate.

$ARGUMENTS
