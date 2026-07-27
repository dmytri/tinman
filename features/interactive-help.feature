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
    When the operator types "how do I record a session" at the assistant prompt without sending
    Then the region titled "Ask Tinman" shows "how do I record a session"

  Rule: the transcript scrolls and the input box stays put. Both halves of the exchange are written into the terminal's own scrollback above the box, the question as it was sent and then the answer, the way a coding agent writes its turns. The exchange accumulates where an operator already knows how to scroll and search, and it survives the session as ordinary terminal output. The box holds only what is being typed: a reply written into the field being edited leaves the operator unable to tell their draft from the program's output, and makes the next keystroke ambiguous.

  Rule: the question is styled apart from the answer. A transcript of alternating turns in one uniform style makes the operator re-read each line to learn whose it is, and the answer is the half they came for. Style carries no meaning here beyond authorship, so a terminal without colour loses only the convenience.

  Scenario: a sent question is written above the input box
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then "what does inspect do" appears above the region titled "Ask Tinman"

  Scenario: the input box is empty once the question is sent
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the region titled "Ask Tinman" shows ""

  Scenario: a question is drawn in a different colour from its answer
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then "what does inspect do" is drawn in a different colour from "Inspect prints the terminal object model of a running program."

  Scenario: an answer is written above the input box rather than into it
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then "Inspect prints the terminal object model of a running program." appears above the region titled "Ask Tinman"
    And the region titled "Ask Tinman" does not show "Inspect prints the terminal object model of a running program."

  Scenario: an earlier exchange stays on screen above the input box
    Given inference is available
    And the assistant answers "Record captures a live session into an editable plan."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does record do" at the assistant prompt
    When the operator types "what does inspect do" at the assistant prompt
    Then "Record captures a live session into an editable plan." appears above the region titled "Ask Tinman"
    And the region titled "Ask Tinman" is the lowest region on the screen

  Rule: the assistant remembers the session it is having. A question is rarely the whole question: an operator asks what a command does and then asks about the thing they were actually trying to do, and an assistant that forgets the first makes them restate it every turn.

  Rule: the session compacts continuously rather than dropping turns at a limit. Context spent on old turns is context and latency spent on every later turn, and this path already waits tens of seconds, so the transcript is kept small from the first turn rather than allowed to grow until a threshold trips. The seventeen most recent exchanges are carried whole, within a transcript budget of 120000 characters. Older ones keep their question and lose their answer, which holds what the session was about while shedding most of what it cost. Only when the transcript still exceeds that budget does the oldest question go, so forgetting is the last resort rather than the mechanism. The budget covers the transcript alone: the bundled skill is fixed and dwarfs it, so budgeting the whole request would be budgeting a constant.

  Rule: compaction is mechanical, never a second model call. Summarising a transcript with the provider would double the latency of the slowest thing Tinman does, on every turn, to save tokens on a request that is mostly the bundled skill anyway. Dropping an answer and keeping its question needs no model and cannot fail.

  Rule: the assertions below read the request Tinman builds, not the reply a model gives. Whether the model uses what it was sent is the model's behaviour, and the @inference tier never asserts that; whether Tinman sent it is Tinman's seam and is checked without spending a call.

  Scenario: a new session carries no earlier exchange
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does record do" at the assistant prompt
    Then the assistant request carries no earlier exchange

  Scenario: a follow-up question carries the exchange before it
    Given inference is available
    And the assistant answers "Record captures a live session into an editable plan."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does record do" at the assistant prompt
    When the operator types "and how do I run the plan it wrote" at the assistant prompt
    Then the assistant request carries the earlier question "what does record do"
    And the assistant request carries the earlier answer "Record captures a live session into an editable plan."

  Scenario: an exchange past the whole window keeps its question and loses its answer
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator has asked "first question" and seventeen questions since
    When the operator types "the nineteenth question" at the assistant prompt
    Then the assistant request carries the question "first question"
    And the assistant request carries no answer for "first question"
    And the assistant request carries seventeen whole exchanges

  Scenario: one question sends exactly one request
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does record do" at the assistant prompt
    When the operator types "and how do I run the plan it wrote" at the assistant prompt
    Then the provider received exactly two assistant requests

  Rule: the cursor sits where the next character will land. A cursor parked outside the box, or left behind while the text grows, tells the operator the program is not listening to them; it is the first thing an operator checks and the last thing a screenshot shows.

  Scenario: the cursor follows what the operator types
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "record" at the assistant prompt without sending
    Then the cursor is inside the region titled "Ask Tinman"
    And the cursor is one column past the "record" it shows

  Rule: a real model call takes tens of seconds, so silence reads as a hang. The wait is reported with something that visibly advances and with the time already spent, because a mark that only spins says the program is alive while an operator deciding whether to wait needs to know how long it has been. The same rule makes the wait escapable: a call that cannot be abandoned holds the terminal for its whole ceiling.

  Scenario: a pending answer reports how long it has been waiting
    Given inference is available
    And the inference provider endpoint accepts the connection and never answers
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the region titled "Ask Tinman" shows the elapsed seconds of the pending call
    And the reported elapsed seconds advance while the call is pending

  Scenario: escape abandons a pending answer and keeps the session
    Given inference is available
    And the inference provider endpoint accepts the connection and never answers
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does inspect do" at the assistant prompt
    When the operator presses "esc" at the assistant prompt
    Then the region titled "Ask Tinman" is drawn
    And the command has not exited

  Rule: the operator can correct what they typed. A prompt that only appends makes a typo unrecoverable, so an operator who mistypes must send the wrong question and ask again, which on this path costs a real model call. Text arrives as bytes and is shown as characters: a character outside ASCII is several bytes, and appending them one at a time renders one replacement mark per byte rather than the character the operator typed.

  Scenario: a typed character can be erased
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "recrd" at the assistant prompt without sending
    And the operator presses "backspace" at the assistant prompt
    Then the region titled "Ask Tinman" shows "recr"

  Scenario: a character outside ASCII is shown as the operator typed it
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "café" at the assistant prompt without sending
    Then the region titled "Ask Tinman" shows "café"

  Rule: the box is bounded rather than stretched, and the bounds are a scantling rather than prose. A prompt spanning a wide terminal puts the text the operator is reading and the cursor they are typing at opposite ends of the screen, and a border drawn edge to edge reads as a rule across the terminal rather than as a box. Tinman's own interface is declared in the terminal object model Tinman builds from every program it drives, so the assistant is checked exactly as a test author checks their own program, and the model is exercised against a real screen on every run. A property the model cannot carry is a finding about the model, not a gap to route around, which is why colour and cursor position keep their own scenarios below.

  @contract
  Scenario: the assistant interface conforms to its terminal object model contract
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal 120 columns wide
    Then the terminal object model of the screen conforms to the "assistant-ui" schema in "scantlings/assistant-ui.schema.json"

  @contract
  Scenario: the assistant interface conforms to its contract on a narrow terminal
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal 40 columns wide
    Then the terminal object model of the screen conforms to the "assistant-ui" schema in "scantlings/assistant-ui.schema.json"
    And the region titled "Ask Tinman" is at most 40 columns wide

  Rule: the border is drawn with rounded corners. A square-cornered box is the default every terminal program has drawn since curses, and the corner glyph is the whole difference between a box that looks considered and one that looks unstyled. It is asserted by the glyph rather than by eye, because a border style is exactly the kind of change a later refactor drops without any test noticing.

  Scenario: the box is drawn with rounded corners
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the region titled "Ask Tinman" has the corner glyph "╭"

  Rule: the key hints say what the keys do now, not what they usually do. While a call is pending, escape abandons that call rather than leaving the session, so a hint reading "esc to leave" would be advertising the wrong outcome at exactly the moment an operator reaches for it. Advertising a key that does something else is the same fault as advertising a command that does nothing.

  Scenario: the hint offers to cancel while an answer is pending
    Given inference is available
    And the inference provider endpoint accepts the connection and never answers
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the assistant prompt names "esc" as the key that cancels

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
