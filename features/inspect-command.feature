Feature: inspect command
  As a test author
  I want to see the terminal object model of a running program
  So that I can discover the roles and names my test should address

  Rule: inspect is the command a test author points at an unfamiliar program, which is exactly the program whose behaviour nobody has read yet. Every scenario here launches one, so every scenario here carries the sandbox tier's cost.

  @sandbox
  Scenario: inspect lists the roles and names on the current screen
    When the operator inspects the fixture terminal program
    Then the inspect output lists a "menuitem" named "Settings"

  @sandbox
  Scenario: inspect emits the model as JSON when asked
    When the operator inspects the fixture terminal program as JSON
    Then the inspect output conforms to the "tom" schema in "scantlings/tom.schema.json"

  @sandbox
  Scenario: inspect reports a program that draws nothing
    When the operator inspects the command "true"
    Then the inspect output reports "no regions on screen"

  Rule: a program that exits on its own has finished speaking, so its whole output is there to read and taking only the last screenful throws the rest away. A program still running has not finished, so what it has drawn is a screen. Exiting is observable, so the distinction needs no flag and no guessing. It matters most where it is least visible: reading a plain command's output as a screen makes an assertion depend on the height of whichever terminal happened to run it, which is the same quiet-wrong-answer class as a locator binding to the wrong region.

  @sandbox
  Scenario: output longer than the terminal is read whole
    When the operator inspects a command printing 200 numbered lines in a terminal 24 rows high
    Then the inspect output carries the line numbered 1
    And the inspect output carries the line numbered 200

  Rule: a stream's shape decides how it reads, the same way a rendered screen's idioms decide how it reads. Output of one item per line is a list to anyone looking at it and to anyone hearing it read aloud, and output of blank-line separated blocks is a sequence of entries. One rule for both is wrong for one of them: either every line of a compiler's diagnostics becomes an item, including the indented continuation that is not one, or every filename from a listing collapses into a single entry and the test author is back to matching raw text. The discriminating block is a multi-line one, so a stray blank line between filenames leaves a listing a listing.

  @sandbox
  Scenario: one item per line is read as a list
    When the operator inspects a command printing "Cargo.toml", "README.md" and "src" on their own lines
    Then the inspect output lists a "list" holding 3 "listitem" regions
    And the inspect output lists a "listitem" named "src"

  @sandbox
  Scenario: blank-line separated multi-line blocks are read as a log of articles
    When the operator inspects a command printing two two-line blocks separated by a blank line
    Then the inspect output lists a "log" holding 2 "article" regions

  @sandbox
  Scenario: a blank line between single lines leaves a listing a list
    When the operator inspects a command printing "src", a blank line and "tests"
    Then the inspect output lists a "list" holding 2 "listitem" regions

  Rule: columnar output is a table, and reading it as a list hands the test author back the raw text this feature exists to spare them. `ls -la` becomes one item per line whose name is the entire line, so asserting a file's size or counting directories means matching a substring of "-rw-r--r--  1 1000 1000    669 Jul 25 14:51 .env". The list rule above already names that failure as the one it prevents; it prevents it for a bare listing and walks into it the moment a column appears. `ps`, `df` and `git log --oneline` are the same shape, and the standing preference that Tinman stay useful for plain command-line testing makes this the half that matters most.

  Rule: a header row names the columns and its absence does not stop them being columns. Where the first row labels the rest, cells are addressed by that label; where nothing labels them, they are addressed by position. WAI-ARIA permits a table with no column headers, so the model reports what it can see and invents no name it cannot, which is the same rule that governs naming everywhere else here.

  Rule: alignment that means columns is alignment sustained across rows, not spacing that happened to line up once. Prose breaks into words at whatever positions it likes, and a paragraph read as a table is wrong in the direction nobody notices, because it still produces regions and still looks addressable. The negative control below is what keeps the discriminator honest; without it a rule that turns everything into a table passes every positive case.

  @sandbox
  Scenario: output whose first row labels the rest is read as a table
    When the operator inspects a command printing "NAME   SIZE", "alpha  10" and "beta   20" on their own lines
    Then the inspect output lists a "table" holding 2 "row" regions
    And the inspect output lists a "columnheader" named "SIZE"

  @sandbox
  Scenario: a cell is addressed by the column that labels it
    When the operator inspects a command printing "NAME   SIZE", "alpha  10" and "beta   20" on their own lines
    Then the "SIZE" cell of the row naming "beta" is "20"

  @sandbox
  Scenario: columns with no header row are addressed by position
    When the operator inspects a command printing "alpha  10" and "beta   20" on their own lines
    Then the inspect output lists a "table" holding 2 "row" regions
    And the second cell of the row naming "alpha" is "10"

  Rule: a column boundary is a character position blank in every row, not a gap of some chosen width. Real listings separate their fields with a single space, right-align their numbers and vary their field widths, so a rule keyed on separator width reads `ls -la` as prose and toy two-space examples as tables. Captain wrote those toy examples and the first implementation satisfied them without ever meeting real output, which is the third fixture this voyage that was easier than the thing it stood for. The data below is shaped like a real listing for that reason.

  @sandbox
  Scenario: a listing whose fields are single-space separated is still a table
    When the operator inspects a command printing "-rw-r--r-- 1 dk dk  669 Jul 25 14:51 .env" and "drwxr-xr-x 2 dk dk 4096 Jul 25 11:38 assets" on their own lines
    Then the inspect output lists a "table" holding 2 "row" regions
    And the fifth cell of the first row is "669"

  @sandbox
  Scenario: prose that happens to align is not read as a table
    When the operator inspects a command printing "the quick brown fox" and "jumped over  the lazy dog" on their own lines
    Then the inspect output lists a "list" holding 2 "listitem" regions
    And the inspect output lists no "table" region

  @sandbox
  Scenario: a running program's table is read as one
    When the operator inspects "top"
    Then the inspect output lists a "columnheader" named "COMMAND"
    And the inspect output lists a "table" holding at least 1 "row" region

  Rule: a --help text, a man page, a tldr page and an inference engine all produce claims about what a program does, and a claim is the one thing Tinman does not accept. The whole thesis of the project is that the screen is the evidence, so a documented example is a hypothesis to run rather than a fact to repeat. Tinman collaborates with these sources in both directions: it reads them for hypotheses here, and it emits its own through `tinman man` and `tinman completions` so other tools can read Tinman the same way. This is also the distance between Tinman and the projects tldr-pages names for putting its pages through a language model and publishing the result: those pass a claim along about commands nobody executed, where inspection runs the command and reports what happened. Documentation that has drifted from its binary announces itself here instead of being quietly believed.

  Rule: the sources rank by how close they sit to the binary in hand. A program's own --help came out of the binary being tested, at the version installed, so it is ground truth and it is always there. A man page is read where the sandbox has one, which a minimal bwrap sandbox does not, and widening the sandbox to read documentation is not worth it. A tldr page describes some version of some machine's program and is a hint only. The ranking decides disagreements rather than tastes: nothing outranks what the binary says about itself.

  Rule: trying a claim is only safe because it is sandboxed, so the two are one requirement rather than two. Documented examples are arbitrary command lines written by strangers, and plenty of them delete things by design. Running them against the operator's real home would be reckless in exactly the way running an untrusted coding agent is, which is the threat Tinman already answers. A probe gets the isolation every other launch gets, so an example that destroys its working tree destroys a copy nobody will miss.

  @sandbox
  Scenario: inspection runs the examples the program's own help carries
    Given the fixture terminal program's help carries the example "fixture --menu"
    When the operator inspects the fixture terminal program with its documented examples
    Then the inspect output lists a "menuitem" named "Settings"
    And the inspect output reports the examples the program's own help carries

  @sandbox
  Scenario: inspection runs the examples its tldr page carries
    Given the configured tldr page source has a page for "printf" carrying the example "printf hello"
    When the operator inspects "printf" with its documented examples
    Then the inspect output lists a region named "hello"

  @sandbox
  Scenario: the binary's own help outranks its tldr page where they disagree
    Given the configured tldr page source has a page for "printf" carrying the example "printf from-the-page"
    And the "printf" help carries the example "printf from-the-binary"
    When the operator inspects "printf" with its documented examples
    Then the inspect output reports "printf from-the-binary" as the ground-truth example
    And the inspect output reports "printf from-the-page" as a hint

  @sandbox
  Scenario: a documented example the program refuses is reported as an unhonoured claim
    Given the configured tldr page source has a page for "ls" carrying the example "ls --wormhole"
    When the operator inspects "ls" with its documented examples
    Then the inspect output reports the example "ls --wormhole" was refused

  @sandbox
  Scenario: a documented example cannot write outside the sandbox
    Given a sentinel path outside the sandbox where no file exists
    And the configured tldr page source has a page for "sh" carrying an example that writes to the sentinel path and prints "ran"
    When the operator inspects "sh" with its documented examples
    Then the inspect output lists a region named "ran"
    And no file exists at the sentinel path

  Rule: Tinman is a testing tool rather than a command explainer, so the product of running a program's documented examples is a plan and not a verdict. An explainer tells the operator what a command does and is believed or not; a probe that writes a plan hands them evidence they commit, re-run, and watch go red the day the program changes under its own documentation. This is what the two phases already promise, reached from a new starting point: the documentation supplies the hypotheses where a human supplied them under `record`, and replay stays deterministic either way. Plan writing is `record`'s seam and `features/record-command.feature` owns it; what is new here is where the steps come from.

  Rule: only an example the program honoured earns an expectation. Writing a refused example into a plan as though it had passed would manufacture the exact fault this project has paid for more than any other, a green assertion over something that never happened, and writing it as an expectation that fails would hand the operator a permanently red plan describing someone else's stale documentation. It is reported instead, which is the finding the operator actually wants: the docs and the binary disagree.

  @sandbox
  Scenario: probing a program's documentation writes a replayable plan
    Given the configured tldr page source has a page for "printf" carrying the example "printf hello"
    When the operator inspects "printf" with its documented examples and writes a plan
    Then the written plan names the command "printf hello"
    And the written plan carries an expectation on the text "hello"
    And replaying the written plan reproduces the recorded interaction

  @sandbox
  Scenario: a refused example is reported rather than written as an expectation
    Given the configured tldr page source has a page for "ls" carrying the example "ls --wormhole"
    When the operator inspects "ls" with its documented examples and writes a plan
    Then the inspect output reports the example "ls --wormhole" was refused
    And the written plan carries no expectation for "ls --wormhole"
    And the written plan is not empty

  Rule: the page is fetched as raw markdown from the tldr project rather than read out of an installed client. Declining to be a client meant declining the client specification, which carries required flags, platform resolution, language handling and cache maintenance; it never meant declining a single request for a single file. Fetching directly also removes a dependency on the operator having a client at all, and removes the raw-output extension that no client is obliged to provide. Markdown is the form the style guide specifies, so the placeholders and the keypress notation arrive as written rather than as somebody's rendering of them. Platform is resolved by asking for the platform page and falling back to the common one, which is two attempts and not a cache.

  @sandbox
  Scenario: a page the project does not carry reports no page
    Given the fixture terminal program's help carries the example "fixture --menu"
    And the configured tldr page source has no page for the fixture terminal program
    When the operator inspects the fixture terminal program with its documented examples
    Then the inspect output reports no tldr page was read
    And the inspect output lists a "menuitem" named "Settings"

  Rule: a tldr example is a template rather than a command line. Its style guide fixes the syntax, so "{{path/to/file}}" and "{{option1|option2}}" and "{{1..5}}" are holes a reader is expected to fill, and passing one to the program as written hands it an argument nobody meant. That failure would arrive wearing the costume of a finding: the program rejects the line, and inspection reports a documented example the program would not honour, which is this project's signature fault manufactured out of nothing but a template Tinman filled in wrong.

  Rule: a page is somebody else's work under CC-BY-4.0, so a command line Tinman assembled is Tinman's line and not the page's. Recording the substituted line as though the page carried it credits the project with words it never wrote, which is the same drift the ranking rule above guards in the other direction. What Tinman ran and what it read are two facts and the plan carries both.

  @sandbox
  Scenario: a placeholder Tinman can fill is filled before the example runs
    Given the configured tldr page source has a page for "cat" carrying the example "cat {{path/to/file}}"
    When the operator inspects "cat" with its documented examples
    Then the inspect output lists a region named "tinman fixture"

  @sandbox
  Scenario: the plan records the line Tinman ran and the example it came from
    Given the configured tldr page source has a page for "cat" carrying the example "cat {{path/to/file}}"
    When the operator inspects "cat" with its documented examples and writes a plan
    Then the written plan names the command Tinman ran with its substituted path
    And the written plan records "cat {{path/to/file}}" as the documented example it came from

  @sandbox
  Scenario: a placeholder Tinman cannot fill leaves the example untestable as written
    Given the configured tldr page source has a page for "printf" carrying the example "printf hello"
    And that page also carries the example "printf {{format}} {{arguments}}"
    When the operator inspects "printf" with its documented examples and writes a plan
    Then the inspect output reports "printf {{format}} {{arguments}}" as untestable as written
    And the written plan carries no expectation for "printf {{format}} {{arguments}}"
    And the written plan is not empty

  @sandbox
  Scenario: a plan whose examples came from a tldr page credits the project
    Given the configured tldr page source has a page for "printf" carrying the example "printf hello"
    When the operator inspects "printf" with its documented examples and writes a plan
    Then the written plan credits the tldr-pages project for the page it read
    And the written plan names "CC-BY-4.0" as that page's licence

  @sandbox
  Scenario: a plan whose examples came only from the binary credits no page
    Given the configured tldr page source has no page for "printf"
    And the "printf" help carries the example "printf hello"
    When the operator inspects "printf" with its documented examples and writes a plan
    Then the written plan credits no page

  Rule: a documented example is not always a command line. The tldr-pages style guide fixes a keypress syntax, "<Ctrl c>" and "<Esc><u>" and "<ArrowUp>", so a page for a full-screen program already documents the interactions rather than only the invocation. That is the half of a TUI's documentation nobody can test by running a shell, and it is the half Tinman exists for: a documented keypress is a plan step in a notation somebody else already standardized. Sending it as text rather than as a key would type the literal angle brackets into the program, which is the placeholder fault again in a different costume.

  @sandbox
  Scenario: a documented keypress example is written as a key step
    Given the fixture terminal program's help carries the example "fixture --menu"
    And that help carries the keypress example "<Down>"
    When the operator inspects the fixture terminal program with its documented examples and writes a plan
    Then the written plan carries a key step sending "Down"
    And replaying the written plan reproduces the recorded interaction

  @sandbox
  Scenario: a keypress notation Tinman cannot send is reported as untestable
    Given the fixture terminal program's help carries the example "fixture --menu"
    And that help carries the keypress example "<Meta Hyper z>"
    When the operator inspects the fixture terminal program with its documented examples and writes a plan
    Then the inspect output reports "<Meta Hyper z>" as untestable as written
    And the written plan carries no key step for "<Meta Hyper z>"
    And the written plan is not empty

  Rule: presentation is half of what a test author is deciding whether to assert on, and one who cannot see from the discovery tool that the status line is red will not write the expectation that pins it. Naming a colour only where a region carries one of its own is what keeps the listing of an ordinary screen as short as it is now.

  @sandbox
  Scenario: the listing reports the colour a region is drawn in
    When the operator inspects a command that prints "disk full" in red
    Then the inspect output lists a region named "disk full"
    And the inspect output reports that region is drawn in red

  Rule: no command a person can run invokes inference. The inference engine is built, specified and green, and every scenario covering it reaches the seam through verification rather than through a command, so the enrichment it produces is unreachable from the product. The bundled skill and the README both promise that capture time may infer, which makes the shipped binary contradict its own documentation. This is the shape a declared command that did nothing produced once already: scenarios green over a seam no user can reach.

  @captain @inference
  Scenario: inspect enriches the deterministic model when a credential is configured
    Given an inference credential is configured
    When the operator inspects the fixture terminal program
    Then the inspect output names a region the deterministic pass leaves unnamed

  Rule: a status region carries its content as text and no name, and the listing prints a region's name, so a status region prints as a bare role with nothing beside it. An operator reading that learns a region exists and nothing about what it says, which is barely better than the region being absent. This is what top's summary line looks like today.

  @captain
  Scenario: the listing shows the text a status region carries
    Given a virtual screen whose bottom line reads "NORMAL  main  3 files"
    When the operator inspects that screen
    Then the inspect output shows the status region carries "NORMAL  main  3 files"
