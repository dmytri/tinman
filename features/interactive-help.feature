Feature: interactive help
  As an operator
  I want to ask Tinman for the command I need
  So that I can act without reading the whole manual

  Rule: the assistant has a deliberately narrow scope. It answers questions about Tinman and proposes Tinman commands. Every action it proposes runs through Tinman's normal command parser, so it can never become a general-purpose shell.

  Rule: the assistant draws an inline box rather than taking the screen. A full-screen prompt would clear the conventional help the operator just asked for, and scroll it out of reach; an inline viewport leaves that output in the scrollback and claims only the rows beneath it. The box is the same shape Tinman reads in the programs it drives, a bordered region carrying a title, so the assistant is legible to Tinman's own model.

  Scenario: the assistant box is drawn beneath the conventional help
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the conventional help is still on the screen
    And a bordered region titled "Ask Tinman" is drawn beneath it

  Scenario: what the operator types is shown in the box
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "how do I record a session" at the assistant prompt
    Then the region titled "Ask Tinman" shows "how do I record a session"

  Rule: the answer appears inside the box that asked for it. An answer printed beneath the box separates the question from its reply by a border and leaves the operator reading two places at once, and it breaks the inline viewport by scrolling the box away from the text it produced. One region holds the exchange.

  Scenario: a question typed at the prompt is answered inside the box
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the region titled "Ask Tinman" shows "Inspect prints the terminal object model of a running program."

  Rule: the operator can correct what they typed. A prompt that only appends makes a typo unrecoverable, so an operator who mistypes must send the wrong question and ask again, which on this path costs a real model call. Text arrives as bytes and is shown as characters: a character outside ASCII is several bytes, and appending them one at a time renders one replacement mark per byte rather than the character the operator typed.

  Scenario: a typed character can be erased
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "recrd" at the assistant prompt
    And the operator presses "backspace" at the assistant prompt
    Then the region titled "Ask Tinman" shows "recr"

  Scenario: a character outside ASCII is shown as the operator typed it
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "café" at the assistant prompt
    Then the region titled "Ask Tinman" shows "café"

  Rule: the box is bounded rather than stretched. A prompt spanning a wide terminal puts the text the operator is reading and the cursor they are typing at opposite ends of the screen, and a border drawn edge to edge reads as a rule across the terminal rather than as a box. It is capped so a wide terminal gets a box, and it yields to a narrow one so the border never wraps. Both ends are asserted: a cap with no floor overflows the narrow case, and a floor with no cap is what stretching already looked like.

  Scenario: the box is capped on a wide terminal
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal 120 columns wide
    Then the region titled "Ask Tinman" is 80 columns wide

  Scenario: the box yields to a narrow terminal
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal 40 columns wide
    Then the region titled "Ask Tinman" is at most 40 columns wide

  Rule: colour marks the box without carrying meaning, so an operator who cannot see it loses nothing. NO_COLOR is honoured because it is the convention every other terminal program already answers to, and a program that invents its own switch makes the operator configure it twice.

  Scenario: the box is drawn in colour
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the region titled "Ask Tinman" is drawn in a colour other than the default foreground

  Scenario: NO_COLOR draws the box without colour
    Given inference is available
    And the environment sets "NO_COLOR" to "1"
    When the operator runs "tinman --help" in an interactive terminal
    Then a bordered region titled "Ask Tinman" is drawn beneath it
    And no cell is drawn in a colour other than the default foreground

  Rule: the prompt names the keys that work it, because an operator dropped into a prompt has no other way to learn them and a terminal offers no menu to discover. The keys are asserted by name here rather than only through the asset body, since a scenario comparing output to an asset passes just as well when the asset loses the line.

  Scenario: the assistant prompt names the keys that send and leave
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the assistant prompt names "enter" as the key that sends
    And the assistant prompt names "esc" as the key that leaves

  Scenario: the assistant session ends when the operator ends the input
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator ends the input
    Then the command exits with status 0

  Scenario: the assistant session ends when the operator presses escape
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator presses "esc" at the assistant prompt
    Then the command exits with status 0

  Scenario: a proposed command is displayed before it runs
    Given the assistant infers the command "tinman record opencode"
    When the operator asks "record the opencode agent"
    Then the assistant displays the proposed command "tinman record opencode"
    And the proposed command has not run

  Scenario: a declined proposal does not run
    Given the assistant has proposed the command "tinman record opencode"
    When the operator declines the proposal
    Then the proposed command has not run

  Scenario: a confirmed proposal runs through the command parser
    Given the assistant has proposed the command "tinman record opencode"
    When the operator confirms the proposal
    Then the command parser receives the arguments "record" and "opencode"

  Scenario: a proposal outside Tinman's command set is refused
    Given the assistant infers the command "rm -rf /"
    When the operator asks "delete everything"
    Then the assistant refuses the proposal
    And no command is offered to the operator

  Scenario: a proposal naming an unknown Tinman subcommand is refused
    Given the assistant infers the command "tinman teleport opencode"
    When the operator asks "teleport the opencode agent"
    Then the assistant refuses the proposal

  Scenario: a question is answered without proposing a command
    Given the assistant answers "Inspect prints the terminal object model of a running program."
    When the operator asks "what does inspect do"
    Then the assistant displays the answer "Inspect prints the terminal object model of a running program."
    And no command is offered to the operator

  @contract
  Scenario: the assistant discharges the command-parser boundary contract
    Given the interactive assistant source
    When the verifier checks the assistant command boundary
    Then no counterexample is found
