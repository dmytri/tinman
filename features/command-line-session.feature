Feature: command-line session
  As an operator exploring a terminal program
  I want to drive it one command at a time against a session I name
  So that I can look around between steps instead of rerunning a whole flow

  Rule: the command line asserts and a plan drives, so the only step verbs here are the ones exploration cannot do without. An expectation is the whole point of the surface. A keypress is the least a person needs to reach a second screen, and without it a session can only ever look at the one the program opened on. Everything richer, activating a region, filling a textbox, capturing a pane, is a sequence, and a sequence wants a file: a plan expresses it, is already committed and replayable, and reaches this surface through the after option rather than as a second spelling.

  Rule: launch and close are the session's own lifecycle rather than steps, which is why neither appears in a written plan.

  Rule: the operator names the session and Tinman never mints a name. A tool that prints an identifier makes the operator capture standard output before they can act, and it makes one value positional while every other is a flag. A named session is deterministic, scriptable, and the same word on every command that touches it.

  @sandbox
  Scenario: a launched session takes the name the operator gave it
    When the operator executes "tinman launch top --session work"
    Then the command exits with a zero status
    And a session named "work" is running

  @sandbox
  Scenario: a verb addresses the session the operator names
    Given a session named "work" running "top"
    When the operator executes "tinman expect PID --session work"
    Then the command exits with a zero status
    And the session named "work" is still running

  Rule: a one-shot command is the general form with the session left out, rather than a second mode with rules of its own. Absent a session the verb launches the program, runs the step and closes, which is the whole of what an operator wants when they are asking one question. Naming both behaviours in one rule is what stops the one-shot path acquiring a different isolation policy or a different failure shape.

  @sandbox
  Scenario: a verb with no session launches and closes around its step
    When the operator executes "tinman expect PID top"
    Then the command exits with a zero status
    And no session is left running

  @sandbox
  Scenario: closing a session ends the program it launched
    Given a session named "work" running "top"
    When the operator executes "tinman close --session work"
    Then the command exits with a zero status
    And no session is left running

  Rule: a session the operator did not launch is an error rather than an implicit launch, because guessing which program they meant is the same guess the driver protocol already refuses when it declines to default a session. A typo in a session name that silently launched a second program would leave two running and report success.

  @sandbox
  Scenario: addressing a session that was never launched fails
    When the operator executes "tinman expect PID --session nosuch"
    Then the command exits with a non-zero status
    And the failure reports no session named "nosuch" is running

  Rule: a session outlives the command that launched it, so it is a real resource with a real cost, and this project has already paid for orphaned sandboxes holding bind mounts against directories the operator was editing. A session carries the isolation the plan's defaults describe, and closing it reclaims what launching it created.

  @sandbox
  Scenario: a session launches its program inside the sandbox
    Given the operator's environment carries "TINMAN_SECRET"
    When the operator launches a session against a target that prints whether it read "TINMAN_SECRET"
    Then the target reports the variable was absent

  @sandbox
  Scenario: closing a session reclaims what launching it created
    Given a session named "work" running "top"
    When the operator executes "tinman close --session work"
    Then no sandbox resource created for "work" remains

  Rule: an operator who walks away is the normal case rather than the exceptional one, and a session whose program has gone is holding a sandbox nobody is watching. This project has already accumulated ninety-nine staging directories and a sandbox three days old holding a bind mount against a tree the operator was editing, so closing on the happy path is not the safety net. Reclaiming at launch is, on the same reasoning that reclaims leftover resources at suite start rather than trusting a killed run to tidy up after itself.

  @sandbox
  Scenario: launching reclaims a session whose program has already exited
    Given a session named "stale" whose program has exited
    When the operator executes "tinman launch top --session work"
    Then the command exits with a zero status
    And no sandbox resource created for "stale" remains
