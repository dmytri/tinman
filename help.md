# Addendum: Interactive Help, Bundled Skill, Captain Custody, and Inferred Acronym

Tinman must ship with a **skills.sh-compatible skill**.

The bundled skill is a **human-owned asset** and the **single source of truth** describing Tinman.

It is consumed by both external coding agents and Tinman itself.

Do not duplicate Tinman's purpose, workflows, or usage in separate prompts, FAQs, embedded prompts, or other documentation.

---

# Bundled Skill

Tinman must include a skills.sh-compatible skill that can be installed and used by compatible coding agents without modification.

The skill should describe:

* what Tinman is
* what problems it solves
* major commands
* common workflows
* recommended usage
* examples
* best practices

All durable agent-facing documentation must live in this skill.

---

# Captain Custody

The bundled skill is a **human-owned asset**.

It is under **Captain custody**.

It must not be generated, rewritten, or maintained by inference.

Changes to the skill are durable specification changes and therefore follow the normal Captain workflow.

Other roles or inference may propose improvements, but they must never modify the skill directly.

Inference consumes the skill.

It does not author it.

---

# Conventional Help

`tinman --help` must always display complete conventional CLI help.

The conventional help is authoritative and must remain fully usable without inference.

When stdout is not a TTY, only conventional help should be rendered.

---

# Interactive Help

When running in an interactive terminal **and inference is available**, append a small interactive assistant beneath the conventional help.

Example:

```text
tinman

Terminal Inference for Navigating Model Agent Networks

Record, inspect and test terminal applications and coding agents.

Usage:
  tinman <COMMAND>

Commands:
  record
  replay
  test
  inspect
  help

Options:
  -h, --help
  -V, --version

Ask Tinman:
> _
```

The interactive assistant has a deliberately narrow scope.

It may:

* answer questions about Tinman
* explain commands
* explain options
* explain workflows
* infer Tinman commands from natural language
* display the proposed command
* ask for confirmation
* execute the confirmed command through Tinman's normal command parser

It must never:

* execute arbitrary shell commands
* become a general-purpose chatbot
* bypass Tinman's existing CLI

All inferred actions must ultimately execute through Tinman's normal command parser.

---

# Tinman Dogfoods Its Own Skill

The interactive assistant inside `tinman --help` must answer questions by loading the bundled skills.sh-compatible skill into the inference context.

Tinman should consume the exact same skill that an external coding agent would consume.

Do not implement a separate help prompt.

Do not implement an embedded FAQ.

Do not implement a second knowledge base.

The bundled skill is the authoritative documentation.

This ensures:

* one source of truth
* improvements immediately benefit both users and agents
* `tinman --help` continuously validates the shipped skill
* the bundled skill remains accurate because Tinman depends upon it itself

---

# Inferred Acronym

When inference is available, display an inferred expansion of **TINMAN** immediately beneath the program name.

Example:

```text
tinman

Terminal Inference for Navigating Model Agent Networks
```

The expansion is generated dynamically.

It is **not** canonical.

Different invocations may produce different expansions.

Its purpose is to demonstrate that inference is available while giving Tinman a small amount of personality.

The expansion should be technically plausible, amusing, and relevant to Tinman's purpose.

---

# Acronym Context

The acronym generator should read **only** the `description` field from the bundled skill.

Do not hardcode another product description.

The bundled skill remains the single source of truth.

---

# Acronym Prompt

Equivalent prompt:

```text
Expand TINMAN into a plausible technical acronym.

Context:
<skill description>

Rules:

- exactly six words
- initials must spell TINMAN
- output exactly one line
- no punctuation
- favour concepts relevant to the supplied description
- be highly creative
- do not explain the result
```

Validate the generated expansion before displaying it.

Validation must ensure:

* exactly six words
* initials spell TINMAN
* single line
* no punctuation

Retry once if validation fails.

---

# Inference Unavailable

If inference is unavailable, disabled, or acronym generation ultimately fails, **do not display an acronym**.

Instead, display a short status notice beneath the program name.

Example:

```text
tinman

Inference unavailable.

Record, inspect and test terminal applications and coding agents.

Usage:
  tinman <COMMAND>

...
```

In this mode:

* display the complete conventional help
* omit the interactive help assistant
* omit acronym generation
* continue to function normally

This is a normal degraded mode and must not be treated as an error.

---

# Inference Profiles

Use separate inference profiles.

## Acronym

The acronym is purely cosmetic.

Optimise for novelty and humour.

Use a **very high temperature**.

## Interactive Help

The interactive assistant generates commands that users may execute.

Optimise for correctness, determinism, and instruction following.

Use a **low temperature**.

---

# Design Principles

The bundled skills.sh-compatible skill has exactly three consumers.

1. External coding agents use the complete skill to understand Tinman.
2. `tinman --help` loads the complete skill to answer user questions.
3. The acronym generator uses only the skill's `description` field as semantic context.

This architecture provides a single, human-owned, Captain-controlled source of truth for Tinman's behaviour.

Tinman should dogfood its own skill wherever practical, while keeping ownership of that skill firmly under Captain custody.

