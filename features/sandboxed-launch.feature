Feature: sandboxed launch
  As a security-conscious operator
  I want a launched program isolated from my home, secrets and network
  So that running an untrusted coding agent is safe by default

  @sandbox
  Scenario: a sandboxed program cannot see the operator's home or secrets
    Given the operator's environment defines the secret "TINMAN_SECRET" as "hunter2"
    And a Bubblewrap-prepared process that prints its home directory and the value of "TINMAN_SECRET"
    When the process is captured through a PTY
    Then the virtual screen shows the secret value is absent
    And the virtual screen shows a home directory other than the operator's home

  @sandbox
  Scenario: a sandboxed program has no network access
    Given a Bubblewrap-prepared process that probes for a network route
    When the process is captured through a PTY
    Then the virtual screen shows the network probe found no route

  Rule: the two scenarios above prepare their own process through the backend, so they reach the backend's treatment of what it prepares and never the commands an operator types. That gap shipped: record and inspect each built a prepared process directly and launched the operator's shell with their real home, environment, path and network, while these scenarios stayed green and the project's own onboarding document described record as sandboxing its target.

  Rule: a scenario asserting that a launch went through the Bubblewrap backend reaches the plumbing being called, and reaches it just as well when the sandbox lets everything through. What separates a contained program from an uncontained one is a file the target was told to write and could not, which is why a sentinel path appears below. An absent file also describes a target that never ran, which is why some output the target drew appears beside it.

  @sandbox
  Scenario: inspect cannot write outside the sandbox
    Given a sentinel path outside the sandbox where no file exists
    When the operator inspects a command that writes to the sentinel path and prints "ran"
    Then the inspect output lists a region named "ran"
    And no file exists at the sentinel path

  @sandbox
  Scenario: record cannot write outside the sandbox
    Given a sentinel path outside the sandbox where no file exists
    When the operator records a command that writes to the sentinel path and prints "ran"
    Then the written plan carries an expectation on the text "ran"
    And no file exists at the sentinel path

  Rule: the sentinel scenarios above guard a path outside the sandbox, and the operator's own working directory was inside it, bound writable and made the launched program's starting directory. So "cannot write outside the sandbox" was true and the operator's tree was still rewritten by anything they probed. An inspected program is an unfamiliar one and a documented example is a command line written by a stranger, which makes the directory the operator is standing in the thing that most needs to survive.

  Rule: an overlay is what lets both promises hold at once. The program reads the real tree through the overlay's lower layer and every write lands in an invisible tmpfs that goes when the sandbox goes, so the feature's existing promise that a destructive example destroys a copy nobody will miss becomes literally true and costs no copy to keep. Replay inherits the same property for free, because every run starts from a tree no earlier run could have altered. Bubblewrap gained these options in 0.11.0.

  @sandbox
  Scenario: inspect cannot write into the operator's working directory
    Given the operator's working directory carries no file named "probe-artifact"
    When the operator inspects a command that writes "probe-artifact" into its working directory and prints "ran"
    Then the inspect output lists a region named "ran"
    And no file named "probe-artifact" exists in the operator's working directory

  @sandbox
  Scenario: record cannot write into the operator's working directory
    Given the operator's working directory carries no file named "probe-artifact"
    When the operator records a command that writes "probe-artifact" into its working directory and prints "ran"
    Then the written plan carries an expectation on the text "ran"
    And no file named "probe-artifact" exists in the operator's working directory

  @sandbox
  Scenario: test cannot write into the operator's working directory
    Given the operator's working directory carries no file named "probe-artifact"
    And a plan whose command writes "probe-artifact" into its working directory and prints "ran"
    When the operator runs that plan
    Then the plan passes
    And no file named "probe-artifact" exists in the operator's working directory

  @sandbox
  Scenario: a sandboxed program reads the operator's working directory
    Given the operator's working directory carries a file "notes.txt" containing "hello from the tree"
    When the operator inspects a command that prints the contents of "notes.txt"
    Then the inspect output lists a region named "hello from the tree"

  Rule: a full-screen program asks the terminfo database what its terminal can do, and answers nothing when there is no answer. The sandbox cleared the environment and restored only HOME and PATH, so TERM was empty, and it bound only /bin, /lib and /lib64, so the database at /etc/terminfo and /usr/share/terminfo was unreachable. Both are needed: a TERM naming a terminal no database describes fails exactly as an unset one does. Tinman exists to drive full-screen programs, so this was the product's own purpose unavailable while every scenario covering it passed.

  Rule: the fixture is why nothing reddened. It writes its own escape sequences and never consults terminfo, so it renders in a sandbox where every real curses program is blind. A fixture that avoids the dependency the real subject has proves the harness rather than the behaviour, which is why the scenarios below drive a program off the operator's own system instead.

  @sandbox
  Scenario: a sandboxed program can resolve its terminal type
    When the operator inspects a command that asks terminfo for the terminal width
    Then the inspect output reports the width rather than an unknown terminal

  Rule: the terminal definition is an input to what a program draws, so taking it from the host makes the host a variable in every recorded plan. A plan captured on one machine would replay differently on another whose database describes the same terminal differently, which is the one promise replay makes. Tinman carries its own definition for the same reason terminal size is a property of the run: the things a program's output depends on belong to Tinman rather than to whoever happened to run it. Carrying it also means the sandbox binds nothing of the host's for this, so the isolation surface shrinks rather than grows. The whole database is far too large to carry, so what is carried is a curated set, and the scenario below is what decides whether the set is sufficient.

  @sandbox
  Scenario: a full-screen program draws a screen Tinman can read
    When the operator inspects "top"
    Then the inspect output lists at least one region

  @sandbox
  Scenario: inspection does not read the host's terminal definitions
    Given the host's terminal database is unavailable
    When the operator inspects "top"
    Then the inspect output lists at least one region

  Rule: an empty screen and a program that never started are different findings, and reporting both as no regions on screen sends the operator looking for a drawing bug in a program that failed to launch. The exit status separates them, and the message names which happened.

  @sandbox
  Scenario: a program that cannot start is reported as failing rather than as silent
    When the operator inspects "definitely-not-a-real-program"
    Then the inspect output reports the program did not start
    And the inspect output does not report no regions on screen

  Rule: Bubblewrap cannot be bundled. It needs unprivileged user namespaces the kernel must already allow, and on many distributions it ships setuid or with file capabilities, none of which a downloaded archive can grant. So it is a system requirement, and the overlay options the workspace depends on arrived in 0.11.0, which makes the floor a version rather than a presence.

  Rule: the floor excludes stock Debian stable and Ubuntu LTS, which ship 0.9.0, and that cost is taken deliberately. Degrading to a read-only workspace where the overlay is missing would put two isolation modes in the field, and scenarios passing under one and not the other is how the working directory stayed writable for a whole voyage without anything reddening.

  Rule: refusing at the moment of execution is too late for a session that executes late. The driver constructs nothing until a launch request arrives, so it would bind a port, accept a caller's connection and let their suite set up before failing; the assistant would open, draw and take a question first. A command that can execute therefore refuses as it starts, while the prepared-process seam refuses anything that reaches it regardless. The two answer different questions, one whether this session can ever work and the other whether this particular launch is sandboxed, so the seam check being unreachable in practice is the negative control working rather than dead code.

  Rule: the refusal is the only text most operators will ever read, so it carries the whole answer: which case it is, since absent means install and too old means upgrade, what was found against what is required, that Tinman never executes outside a sandbox so no flag will lift it, and a link that saves a human or an agent a search. An agent meeting a bare "unavailable" cannot tell a fatal requirement from a bug in Tinman.

  @sandbox
  Scenario: an absent sandbox refuses before anything is executed
    Given the Bubblewrap binary is not on the path
    When the operator inspects the fixture terminal program
    Then the command exits with a non-zero status
    And the failure reports the required Bubblewrap version
    And no program was executed

  @sandbox
  Scenario: a sandbox older than the floor refuses and names both versions
    Given the Bubblewrap binary reports version "0.9.0"
    When the operator inspects the fixture terminal program
    Then the command exits with a non-zero status
    And the failure reports the version found and the version required

  @sandbox
  Scenario: a session that executes late refuses as it starts
    Given the Bubblewrap binary is not on the path
    When the operator starts the driver
    Then the command exits with a non-zero status
    And the driver accepted no connection

  Scenario: the commands that execute nothing still answer without a sandbox
    Given the Bubblewrap binary is not on the path
    When the operator runs "tinman --help" with stdout redirected to a file
    Then the output names every command the parser accepts

  Scenario: the refusal points at instructions that exist
    Given the refusal text in the assets
    When the link it carries is read
    Then it names a heading the file at "README.md" carries

  Rule: the scenario above reads the refusal asset as a document and asserts its link resolves. Nothing asserts an operator ever sees it. The asset at assets/help/sandbox-unavailable.txt carries two placeholders, the required version and the version found, matching the two values the sandbox launcher interpolates into a one-line message of its own. No implementation module reads that asset, so the explanation that no flag lifts the sandbox requirement, the install link, and the note that the commands executing nothing still work reach nobody. The scenario below asserts the copy the operator meets is the copy the catalogue holds.

  Scenario: the refusal an absent sandbox prints is the copy the assets carry
    Given the Bubblewrap binary is not on the path
    When the operator inspects the fixture terminal program
    Then the refusal names the required Bubblewrap version
    And the refusal explains that no flag lifts the sandbox requirement
    And the refusal carries the install instructions link
    And the refusal names the commands that still answer without a sandbox

  Rule: a prepared process is the only input the PTY runner launches, so whoever constructs one settles what isolation the launched program gets. A per-module reference contract reaches the modules someone remembered to write one for, and the modules that bypassed the backend were exactly the ones nobody had. A bound over the whole implementation tree is what leaves a command added later covered by a contract nobody edited.

  @contract
  Scenario: only the sandbox backend constructs a prepared process
    Given the implementation sources
    When the verifier checks the prepared-process construction boundary
    Then no counterexample is found
