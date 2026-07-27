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

  Scenario: completions emits a script for the shell named
    When the operator runs "tinman completions bash" with stdout redirected to a file
    Then the output is a completion script naming "tinman"

  Scenario: completions refuses a shell it cannot emit for
    When the operator executes "tinman completions klingon"
    Then the command exits with a non-zero status

  Rule: the named-commands floor is what makes adding a command safe. A count alone would pass a parser that had lost inspect and gained something else, and an unnamed floor passes a parser that reads nothing at all. Naming the seven is what turns a silent loss into a red.

  Scenario: the parser accepts at least the seven named commands
    Given the commands Tinman names
      | command     |
      | record      |
      | test        |
      | inspect     |
      | driver      |
      | help        |
      | man         |
      | completions |
    When each is passed to the command parser
    Then the parser accepts every one

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
