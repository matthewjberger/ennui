---
name: list-skills
description: The user starts it. List every ennui skill with what it does, when to use it, how to start it, and an example. Use when the user asks which skills exist, what a skill is for, which skill fits a task, or how to start one.
---

# List skills

Give the user one short table of the ennui skills. Read the skills fresh each time, so the list stays true when skills change.

## Steps

1. Read the frontmatter (`name` and `description`) of every ennui skill: `.claude/skills/*/SKILL.md` in this repository (the ennui clone, or an app that `just sync` filled). Include this skill too.
2. For each skill, write one row with these columns:
   - **Skill**: the name, as `/name`.
   - **Who starts it**: "You" or "Claude", from the first sentence of the description ("The user starts it" or "Claude runs it" / "Claude loads it"). When both can, write "Claude (you can too)".
   - **What it does**: one sentence from the first part of the description.
   - **Use it when**: one sentence from the "Use when" part of the description.
   - **Example**: one realistic line the user could type, such as `/audit crates/ui` or `/editor add a volume slider to the settings screen`. Base it on what the skill does, and on real names (apps in `apps/`, screens under `project/scenes/ui/`).
3. Put the skills that the user starts first, then the ones Claude starts; sort by name inside each group.
4. After the table, add these lines:
   - The one entry point for interface work: the skill whose description calls itself the entry point (today `/editor`). Name it.
   - Skills start by themselves when a request matches their description, so typing `/name` is only needed to be sure that one loads. Claude loads the skills in the Claude group on its own; the user never needs to type them.
   - Any text after `/name` goes to the skill as its request.
5. If the user asked about one task, say which skill fits it and why, in one sentence, after the table.

Keep the answer short. Do not describe skills that are not ennui skills, except to say that Claude Code has its own built-in commands, such as `/help`, which this list does not cover.
