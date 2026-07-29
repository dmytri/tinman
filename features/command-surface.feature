Feature: command surface
  As an operator, a packager or a coding agent
  I want Tinman's whole command surface emitted from the parser that enforces it
  So that what I am told to run is what Tinman accepts

  Rule: one surface, three consumers: a human at a terminal, Tinman's own assistant, and someone else's coding agent. Nothing here is assistant-only, so discoverability is the whole of the design. A second list of what the parser accepts is a list that drifts from it, which is why the clap definition is the only place any of this is written down.

  Rule: a generated file in the tree is a fourth surface to drift from the parser, and it owes a conformance check to keep it current. A command that emits it on demand cannot drift, still works after "cargo install" where a man page shipped in the repository does not, and lets a packager pipe the output straight into their build.

  Scenario: man emits a roff man page for tinman
    When the operator runs "tinman man" with stdout redirected to a file
    Then the output begins with a roff title macro naming "tinman"
    And the output names every command the parser accepts

  Rule: a page that opens with a title macro and names every command can still be roff that a formatter rejects, and the reader who discovers that is the packager piping it into a build or the operator at "man tinman". mandoc is the language's own parser, so the check belongs to the standard rather than to a shape assertion of ours.

  @contract
  Scenario: the emitted man page is valid roff
    When the operator runs "tinman man" with stdout redirected to a file
    Then mandoc parses the emitted page and reports no error
    And the emitted page is not empty

  Rule: a doc comment on a command type serves developers, and clap turns it into the description an operator reads. Those are two audiences and the operator gets the wrong one: the NAME line shipped three sentences on why the parser's own help output is disabled and whose version flag it is, where the convention is one short line. Copy for a human is an asset here, so the man page's prose is joined to `assets/help/tinman.txt` rather than written where the compiler happens to read it. The join is what stops the two surfaces drifting, since nothing about a stale man page announces itself.

  Scenario: the man page names Tinman in one line
    When the operator runs "tinman man" with stdout redirected to a file
    Then the NAME section is a single line
    And it carries the one-line description from the asset at "assets/help/tinman.txt"

  Scenario: the man page describes Tinman in the operator's terms
    When the operator runs "tinman man" with stdout redirected to a file
    Then the DESCRIPTION section carries the closing paragraph of the asset at "assets/help/tinman.txt"

  Rule: the man page is generated from the doc comments on the command types, and those doc comments are where the trace annotations live. So the one artifact a packager installs system-wide is the one surface that can print the project's own internal bookkeeping at an operator. It did: the DESCRIPTION carried two `@planks` strings on the voyage that added the command. Nothing else Tinman emits is derived from doc comments, so this is the only surface that needs the guard.

  Scenario: the emitted man page carries no trace annotation
    When the operator runs "tinman man" with stdout redirected to a file
    Then the output names every command the parser accepts
    And the output carries no "@planks" annotation

  Rule: the page's SUBCOMMANDS section cross-references `tinman-record(1)` and six more by name, which is the roff convention for "this page exists, go read it". Seven of them did not exist and nothing could emit them, so the one artifact a packager installs system-wide sent every reader to a page no machine has. A cross-reference is a claim about what is installed, and this project does not ship claims it cannot honour. The join below is what keeps the two in step: the page names the pages, and every name it prints is one the binary will produce.

  Scenario: man emits the page for a named subcommand
    When the operator runs "tinman man record" with stdout redirected to a file
    Then the output begins with a roff title macro naming "tinman-record"

  Scenario: every subcommand page the man page cross-references can be emitted
    Given the subcommand pages the man page cross-references
    When each is requested from "tinman man"
    Then every one is emitted
    And the cross-references read are not empty

  Scenario: completions emits a script for the shell named
    When the operator runs "tinman completions bash" with stdout redirected to a file
    Then the output is a completion script naming "tinman"

  Scenario: completions refuses a shell it cannot emit for
    When the operator executes "tinman completions klingon"
    Then the command exits with a non-zero status

  Rule: probing a program's documentation is a mode of inspection rather than a command of its own, so it is a flag on `inspect` and the named set stays at seven. Writing the result reuses the option `record` already carries, because two commands that both write a plan should not spell it two ways. Naming both here is what stops the operator surface being settled in verification support by whoever authors a step first.

  Scenario: inspect accepts the documented-examples flag
    When "inspect --examples printf" is passed to the command parser
    Then the parser accepts it

  Scenario: inspect writes its plan where the operator says
    When "inspect --examples printf --output plan.yaml" is passed to the command parser
    Then the parser accepts it

  Rule: naming the commands is what makes changing the set safe. A count alone would pass a parser that had lost inspect and gained something else, and a set read from nothing at all passes every parser. The assertion is exact rather than a floor, because a floor is silent about the command nobody meant to ship: an addition has to be written here before it reaches an operator, which is the agreement `features/command-dispatch.feature` states and this scenario is the only place that enforces.

  Scenario: the parser accepts exactly the seven named commands
    Given the commands Tinman names
      | command     |
      | record      |
      | test        |
      | inspect     |
      | driver      |
      | help        |
      | man         |
      | completions |
    When the accepted command set is read
    Then it is exactly the seven commands named

  Rule: documentation is where a reader or an agent learns what to run, and a command struck from the parser stays in the prose through the voyage that struck it. One sweep over every shipped markdown file replaces the per-asset check the bundled skill used to carry: the skill, the readme and the onboarding document all ship, and all three tell someone what to type. A fence reader that matches nothing would report a clean bill for every document it never read, which is what the floor below is for.

  Scenario: every Tinman command line in shipped markdown is accepted by the parser
    Given the Tinman command lines in the fenced blocks of these documents
      | document              |
      | README.md             |
      | AGENTS.md             |
      | assets/skill/SKILL.md |
    When each is passed to the command parser
    Then the parser accepts every command line
    And the command lines read are not empty

  Rule: three commands write output a program reads, the JSON model, the manual page and the completion script, so stdout is a data stream rather than a place to talk to the operator. A diagnostic on stdout lands in the middle of whatever the consumer is parsing. Failures reached both streams: an unavailable sandbox went to stderr from inspect and driver, while a failing plan, a refused recording and a failed inspect were printed to stdout. Each of those three carries its own scenario below, because one fixed path proves nothing about the two beside it and a Rule naming them is context rather than a requirement.

  Scenario: a failure is reported on the error stream rather than the data stream
    Given a harness plan driving the fixture terminal program whose final step expects the text "Deployed"
    When the operator tests that plan with the streams captured separately
    Then the error stream reports the step expecting "Deployed"
    And the data stream carries nothing

  Scenario: a refused recording reports on the error stream
    Given the file "session.yaml" already exists
    When the operator records "printf READY" to that file with the streams captured separately
    Then the error stream reports the file already exists
    And the data stream carries nothing

  @sandbox
  Scenario: a failed inspection reports on the error stream
    Given a command the sandbox cannot execute
    When the operator inspects that command with the streams captured separately
    Then the error stream reports the failure
    And the data stream carries nothing

  Rule: the three scenarios above each prove one path and say nothing about the path beside it, which is how a fourth survived them. It sat in the inspect --examples arms and was found by a command rather than by eye, after all three were green. A bound over the implementation tree is the form that reaches it and reaches the fifth a later command adds, so the scenarios stay as the operator-facing proof and the contract below carries exhaustiveness.

  @contract
  Scenario: no failure reaches the data stream
    Given the implementation sources
    When the verifier checks the diagnostic stream boundary
    Then no counterexample is found
