Feature: command-line arguments
  As an operator driving a real program
  I want the target stated the way a plan states it
  So that I can drive a program that takes flags, and ask Tinman what it accepts

  Rule: the target is one argument, quoted, the same way a plan states it. A plan writes `tui: opencode` and `run: git init` as a single string and splits it itself, so the command line states a target identically and the two surfaces agree. Everything else on the line is Tinman's own: a positional for what it is asking, and options for how.

  Rule: this removes the boundary problem rather than solving it. A trailing list of the target's words leaves nothing marking where Tinman's arguments end, and on 2026-08-03 that cost both directions at once: a target's own flag was refused, so `ls -la` and `git log --oneline` could not be driven at all, and every one of the ten commands answered a request for help by reporting an unexpected argument. With the target quoted there is no trailing list for a flag to fall into.

  Rule: a coding agent is one of this surface's three consumers and the one that cannot ask a person, so discoverability is a requirement rather than a courtesy. The command names are already reachable, joined to the parser from the bundled help and the bundled skill. What no join reaches is a command's own options, and those are what an agent needs in order to call it: reading that `expect` exists says nothing about the role, name, after, conforms, output and session it takes. A surface that cannot describe its own options is one an agent has to guess at, and this project's whole argument is that a guess is not evidence.

  Scenario: every command answers for itself when asked for help
    Given the commands Tinman names
    When each is asked for help
    Then every one reports its own usage
    And none reports an unexpected argument
    And every option a command accepts appears in that command's help

  Rule: the discriminator is the target's own output rather than an exit status. A long listing opens with a total line and a bare one does not, so the expectation holds only where the flag reached the program. Asserting that the command merely ran would pass on a Tinman that dropped the flag silently, which is the shape this project keeps paying for.

  @sandbox
  Scenario: a quoted target carrying its own flags is driven
    When the operator executes "tinman expect total 'ls -la'"
    Then the command exits with a zero status

  Rule: a flag Tinman defines is Tinman's outside the quoted target and the program's inside it. Reading one of its own flag names from within the target would steal an argument the program needed, and a program is entitled to any spelling it likes, including the ones Tinman happens to use.

  @sandbox
  Scenario: a flag Tinman defines is the program's inside the quoted target
    When the operator executes "tinman expect total 'ls --all -l'"
    Then the command exits with a zero status

  Scenario: a flag Tinman defines is Tinman's outside the quoted target
    When "expect --role columnheader --name PID top" is passed to the command parser
    Then the parser accepts it

  Rule: a target of one word needs no quoting, because a shell strips quotes it does not need and an operator asking one question of one program should type one thing. The quoted form and the bare word reach the same program, so nothing about the surface depends on which the operator chose.

  @sandbox
  Scenario: a target of one word needs no quotes
    When the operator executes "tinman expect PID top"
    Then the command exits with a zero status

  Rule: this feature's help scenario runs 72760ms and is the binding constraint on its tier at any worker count, since a tier's wall clock cannot fall below its longest scenario. Its first two assertions need real execution and stay at one process per command. The third, that every option appears in each command's help, is a structural fact clap answers from the same definition that builds the help text, so it needs no process per command. This supersedes that third assertion rather than adding beside it.

  @captain
  Scenario: every command names its options in the manual it emits
    Given the command definitions the parser carries
    When the manual page for each command is read
    Then every command definition was read
    And each one names every option it declares
