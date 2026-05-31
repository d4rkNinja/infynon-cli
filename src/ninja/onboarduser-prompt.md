# User Onboarding Prompt

The user's soul profile is currently blank.

Before doing substantial project work, collect stable global user context and save it to the INFYNON soul profile. This onboarding is required only while the soul profile is empty.

## Goal

Create a useful `soul.md` profile that future agents can read with `infynon soul show` and use to adapt behavior to the user.

## What To Collect

Ask the user for the minimum stable information needed to personalize future work:

- Name: what the user should be called.
- Purpose: what the user is trying to achieve with INFYNON.
- Profession: the user's role, domain, or regular work.
- Current Projects: stable, ongoing projects or products.
- Skills: important technical skills, tools, frameworks, or preferred stack.
- Goals: near-term and long-term goals.
- Communication Style: preferred tone, brevity, structure, and directness.
- Answer Style: how detailed answers should be, and whether code, commands, or summaries are preferred.
- Decision Preferences: how the user wants tradeoffs, risks, and recommendations handled.
- Coding Preferences: stable coding conventions, quality bar, review expectations, testing preferences, and architecture preferences.
- Global Constraints: global restrictions or requirements that apply across workspaces.

Do not ask for secrets, credentials, API keys, private tokens, or sensitive personal information.
Do not store workspace-specific rules in `soul.md`. Workspace-specific information belongs in project files, workspace config, task notes, or task results.
Do not invent missing details.

## Required Workflow

1. Run `infynon soul show`.
2. Confirm the returned `is_blank` value is true.
3. Ask the user concise questions for the missing stable context.
4. Build a clean Markdown profile using this structure:

```markdown
# Soul Profile

## Name

## Purpose

## Profession

## Current Projects

## Skills

## Goals

## Communication Style

## Answer Style

## Decision Preferences

## Coding Preferences

## Global Constraints
```

5. Save the completed profile with:

```bash
infynon soul update --text "<complete markdown profile>"
```

If the content is too large or easier to edit directly, use the `soul_path` returned by `infynon soul show` and write the Markdown profile there.

After the soul profile is saved, continue with the user's original request using the newly saved context.
