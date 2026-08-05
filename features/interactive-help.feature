Feature: interactive help
  As an operator
  I want to ask Tinman for the command I need
  So that I can act without reading the whole manual

  Rule: the assistant has a deliberately narrow scope. It answers questions about Tinman and proposes Tinman commands. Every action it proposes runs through Tinman's normal command parser, so it can never become a general-purpose shell.

  Rule: a tool that reads terminal programs for structure, naming and presentation, and then ships a prompt with none of them, has argued against itself. Tinman's own interface is therefore held to the standard the tool exists to measure, and held to it by the same instrument: the terminal object model Tinman derives from every other program it drives. What follows is that standard made falsifiable, in place of a claim about taste.

  Rule: meaning carried in colour alone is the accessibility failure a terminal interface can readily commit, and the one an operator cannot work around. Contrast belongs to the terminal's theme and wording to the copy, but a distinction drawn only in colour is the program's own doing. The reader the scenarios below are written for is on a monochrome terminal, honouring NO_COLOR, or unable to tell the chosen colours apart.

  @assistant
  Scenario: every region of the assistant interface that needs a name has one
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then every region whose role requires an accessible name carries one
    And the regions read are not empty

  Rule: a colour escape in a pipe is corruption of somebody's data, and the conventional help is compared against its asset byte for byte. The discipline behind every styling decision here is being deliberate about which stream it reaches, rather than spraying it at all of them.

  @assistant
  Scenario: redirected help carries no styling
    Given inference is available
    When the operator runs "tinman --help" with stdout redirected to a file
    Then the help output carries no escape sequence

  Rule: the usual reason to leave the assistant is to run the command it just gave, so an exit that takes the session with it loses the one line the operator was there for. Drawing the session into the main screen instead would scroll the output they asked for out of reach while it runs. The alternate screen answers both at once: the session's redrawing stays off the scrollback, and the transcript joins it once, at the end, as ordinary terminal output.

  @assistant
  Scenario: the assistant box is drawn when help is asked for interactively
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then a bordered region titled "Ask Tinman" is drawn

  @assistant
  Scenario: the conventional help is back on the screen when the assistant leaves
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator presses "esc" at the assistant prompt
    Then the conventional help is on the screen

  Rule: the session runs on the alternate screen so the operator's scrollback is theirs until the assistant leaves, and the transcript is written back into it on exit. Asserting only that the screen carries the text afterwards cannot tell that apart from a session that never switched screens at all, which is what the code does today while this feature's prose has claimed otherwise. Scrollback is meaningful because an alternate screen was left; with no alternate screen there is nothing to write back into.

  @sandbox
  Scenario: the assistant draws on the alternate screen
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the terminal is on the alternate screen

  @sandbox
  Scenario: the operator's scrollback is untouched while the assistant is drawing
    Given inference is available
    And the terminal scrollback carries the line "earlier work"
    When the operator runs "tinman --help" in an interactive terminal
    Then "earlier work" is not on the screen

  @assistant
  Scenario: the transcript is written into the scrollback when the assistant leaves
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does inspect do" at the assistant prompt
    When the operator presses "esc" at the assistant prompt
    Then "what does inspect do" is on the screen
    And "Inspect prints the terminal object model of a running program." is on the screen

  Rule: an operator who types the program's name with nothing after it is asking what it does, and the assistant is the thing that answers that. What decides whether answering it is safe is the shape of the invocation and the streams around it, and stdin is the condition that earns its place: "tinman < plan.yaml" is a file being handed to a program, not an operator waiting at a prompt.

  @assistant
  Scenario: bare tinman opens the assistant
    Given inference is available
    When the operator runs "tinman" in an interactive terminal
    Then a bordered region titled "Ask Tinman" is drawn

  @assistant
  Scenario: bare tinman does not list the commands
    Given inference is available
    When the operator runs "tinman" in an interactive terminal
    Then the screen does not carry the Commands block of the asset at "assets/help/tinman.txt"
    And a bordered region titled "Ask Tinman" is drawn

  Rule: the assistant holds the terminal in raw mode on the alternate screen, so it owns restoring both however it leaves. Restoring on the ordinary path only is the case that never gets tested, because the ordinary path is the one everybody runs; the operator meets the other one, with no prompt, no cursor and no echo, and a terminal they have to reset by hand.

  Rule: the interrupt below is the one abnormal exit a scenario can reach, and the error path is carried by a rule in the verification-conformance set rather than by a scenario beside it. The reason is the observation channel: these scenarios read the terminal back through the shell that launched the program, and once raw mode is entered the only route to an error is a failed write to the terminal itself, which takes the reporting shell with it. A simulated failure would be the forbidden double and an invented trigger would be a scenario against an event the product does not have, so what is checked instead is the structure that makes every path restore, which is decidable where the event is not.

  @sandbox
  Scenario: an interrupted session still gives the terminal back
    Given the assistant is drawing its box
    When the operator interrupts the session
    Then the terminal is out of raw mode
    And the alternate screen has been left

  Rule: an assistant that cannot reach a model has one useful thing to say, which is how to give it one. Drawing the ask box anyway offers an input that cannot answer, so the setup form takes that place instead. It stays deterministic and needs no model to run, which is why it is drawn here rather than added as a command: the operator already has the environment and a dotenv file, so a command would be a third route to a value two routes already reach.

  Rule: the credential is a secret, so it is masked as it is typed and written where a secret belongs. The operator's project directory is the wrong home for it, since a file written beside their work is a file their next commit can carry; the configuration directory is owner-readable and outside any repository. Masking is the model's `hidden` attribute doing the job it names.

  @sandbox
  Scenario: the setup form replaces the ask box when inference is unavailable
    Given no inference credential is configured
    When the operator runs "tinman" in an interactive terminal
    Then a region titled "Set up inference" is drawn
    And no region titled "Ask Tinman" is drawn

  @sandbox
  Scenario: the setup form offers the defaults it would otherwise use
    Given no inference credential is configured
    When the operator runs "tinman" in an interactive terminal
    Then the form offers "https://openrouter.ai/api/v1" as the endpoint
    And the form offers "deepseek/deepseek-v4-flash-0731" as the model

  @sandbox
  Scenario: the key is masked as it is typed
    Given no inference credential is configured
    And the operator has opened the setup form
    When the operator types a key into the credential field
    Then the credential field is hidden

  @sandbox
  Scenario: the form names the environment as the other way in
    Given no inference credential is configured
    When the operator runs "tinman" in an interactive terminal
    Then the form names "TINMAN_API_KEY" as an environment variable it reads

  @sandbox
  Scenario: a saved credential is written where only its owner can read it
    Given no inference credential is configured
    And the operator has opened the setup form
    When the operator saves a key through the form
    Then the credential is written under the configuration directory
    And that file is readable only by its owner
    And no credential is written to the operator's working directory

  @assistant
  Scenario: bare tinman renders the conventional help when no credential is configured
    Given no inference credential is configured
    When the operator runs "tinman" in an interactive terminal
    Then the conventional help is on the screen
    And no bordered region titled "Ask Tinman" is drawn

  @assistant
  Scenario: tinman with stdin redirected renders the conventional help
    Given inference is available
    When the operator runs "tinman" in an interactive terminal with stdin redirected from a file
    Then the conventional help is on the screen
    And no bordered region titled "Ask Tinman" is drawn

  @assistant
  Scenario: tinman with stdout redirected renders the conventional help
    Given inference is available
    When the operator runs "tinman" with stdout redirected to a file
    Then the help output is the asset at "assets/help/tinman.txt" with the tagline line removed

  Rule: the name and the tagline are the program saying what it is, and the block beneath them is reference material. Run together in one colour, the operator reads the whole screen to find where the reference starts.

  @assistant
  Scenario: the name and tagline are drawn apart from the help text
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the name "tinman" is drawn in a colour other than the default foreground
    And the tagline is drawn in a colour other than the default foreground
    And the Commands block is drawn in the default foreground

  Rule: the transcript scrolls and the input box stays put. Both halves of the exchange are written above the box, the question as it was sent and then the answer, the way a coding agent writes its turns. The box holds only what is being typed: a reply written into the field being edited leaves the operator unable to tell their draft from the program's output, and makes the next keystroke ambiguous.

  @assistant
  Scenario: what the operator types is shown in the box
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "how do I record a session" at the assistant prompt without sending
    Then the region titled "Ask Tinman" shows "how do I record a session"

  @assistant
  Scenario: a sent question is written above the input box
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then "what does inspect do" appears above the region titled "Ask Tinman"

  @assistant
  Scenario: the input box is empty once the question is sent
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the region titled "Ask Tinman" shows ""

  @assistant
  Scenario: an answer is written above the input box rather than into it
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then "Inspect prints the terminal object model of a running program." appears above the region titled "Ask Tinman"
    And the region titled "Ask Tinman" does not show "Inspect prints the terminal object model of a running program."

  @assistant
  Scenario: an earlier exchange stays on screen above the input box
    Given inference is available
    And the assistant answers "Record captures a live session into an editable plan."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does record do" at the assistant prompt
    When the operator types "what does inspect do" at the assistant prompt
    Then "Record captures a live session into an editable plan." appears above the region titled "Ask Tinman"
    And the region titled "Ask Tinman" is the lowest region on the screen

  Rule: a line of prose drawn the full width of a wide terminal is hard to read back, and it leaves the start of the next line a long way from the end of the last; it is the reason a book is not typeset across a table. A transcript wrapped at the terminal beside a box capped somewhere else gives one screen two measures, which is the same fault a second time.

  @assistant
  Scenario: the box is no wider than the measure on a wide terminal
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal 120 columns wide
    Then the region titled "Ask Tinman" is at most 72 columns wide

  @assistant
  Scenario: an answer wider than the measure is wrapped to it
    Given inference is available
    And the assistant answers a single line of 200 characters
    And the operator runs "tinman --help" in an interactive terminal 120 columns wide
    When the operator types "what does inspect do" at the assistant prompt
    Then the answer is drawn on more than one line
    And no line of the answer is wider than 72 columns

  Rule: colour laid on the words themselves reads as a highlighter smear over them, where a background running the whole measure reads as a block, and a block is what tells the operator at a glance where their own turn started. Colour carries authorship here and nothing besides. Coloured text degrades to text when colour is off, but a background block degrades to no structure at all, so the uncoloured rendering has to find the boundary somewhere other than colour.

  @assistant
  Scenario: a sent question is drawn as a background block
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then "what does inspect do" is drawn on a background other than the default background
    And that background runs the full measure of the transcript
    And "Inspect prints the terminal object model of a running program." is drawn on the default background

  @assistant
  Scenario: NO_COLOR marks the question with a leading marker instead
    Given inference is available
    And the environment sets "NO_COLOR" to "1"
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the question is drawn with the leading marker "> "
    And no cell is drawn on a background other than the default background

  Rule: the model writes a command line as a fenced block, because that is how a command line is written down. A raw fence is three backticks the operator reads past on the line they came for, and the block inside it is the one thing on the screen they are going to copy.

  @assistant
  Scenario: a fenced block in an answer is rendered rather than printed
    Given inference is available
    And the assistant answers:
      """
      Record it:

      ```
      tinman record opencode
      ```
      """
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "how do I record opencode" at the assistant prompt
    Then "tinman record opencode" appears above the region titled "Ask Tinman"
    And no line on the screen carries a fence marker

  @assistant
  Scenario: a fenced block is drawn apart from the prose around it
    Given inference is available
    And the assistant answers:
      """
      Record it:

      ```
      tinman record opencode
      ```
      """
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "how do I record opencode" at the assistant prompt
    Then "tinman record opencode" is drawn in a different colour from "Record it:"

  Rule: the assistant remembers the session it is having. A question is rarely the whole question: an operator asks what a command does and then asks about the thing they were actually trying to do, and an assistant that forgets the first makes them restate it every turn.

  Rule: the session compacts continuously rather than dropping turns at a limit. Context spent on old turns is context and latency spent on every later turn, and this path already waits tens of seconds, so the transcript is kept small from the first turn rather than allowed to grow until a threshold trips. The seventeen most recent exchanges are carried whole, within a transcript budget of 120000 characters. Older ones keep their question and lose their answer, which holds what the session was about while shedding most of what it cost. Only when the transcript still exceeds that budget does the oldest question go, so forgetting is the last resort rather than the mechanism. The budget covers the transcript alone: the bundled context is fixed and dwarfs it, so budgeting the whole request would be budgeting a constant.

  Rule: compaction is mechanical, never a second model call. Summarising a transcript with the provider would double the latency of the slowest thing Tinman does, on every turn, to save tokens on a request that is mostly bundled context anyway. Dropping an answer and keeping its question needs no model and cannot fail.

  Rule: the assertions below read the request Tinman builds, not the reply a model gives. Whether the model uses what it was sent is the model's behaviour, and the @inference tier never asserts that; whether Tinman sent it is Tinman's seam and is checked without spending a call.

  @assistant
  Scenario: a new session carries no earlier exchange
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does record do" at the assistant prompt
    Then the assistant request carries no earlier exchange

  @assistant
  Scenario: a follow-up question carries the exchange before it
    Given inference is available
    And the assistant answers "Record captures a live session into an editable plan."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does record do" at the assistant prompt
    When the operator types "and how do I run the plan it wrote" at the assistant prompt
    Then the assistant request carries the earlier question "what does record do"
    And the assistant request carries the earlier answer "Record captures a live session into an editable plan."

  @assistant
  Scenario: an exchange past the whole window keeps its question and loses its answer
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator has asked "first question" and seventeen questions since
    When the operator types "the nineteenth question" at the assistant prompt
    Then the assistant request carries the question "first question"
    And the assistant request carries no answer for "first question"
    And the assistant request carries seventeen whole exchanges

  @assistant
  Scenario: one question sends exactly one request
    Given inference is available
    And the assistant answers "Inspect prints the terminal object model of a running program."
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does record do" at the assistant prompt
    When the operator types "and how do I run the plan it wrote" at the assistant prompt
    Then the provider received exactly two assistant requests

  Rule: the cursor sits where the next character will land. A cursor parked outside the box, or left behind while the text grows, tells the operator the program is not listening to them; it is the first thing an operator checks and the last thing a screenshot shows.

  @assistant
  Scenario: the cursor follows what the operator types
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "record" at the assistant prompt without sending
    Then the cursor is inside the region titled "Ask Tinman"
    And the cursor is one column past the "record" it shows

  Rule: a real model call takes tens of seconds, so silence reads as a hang. The wait is reported with something that visibly advances and with the time already spent, because a mark that only spins says the program is alive while an operator deciding whether to wait needs to know how long it has been. The same rule makes the wait escapable: a call that cannot be abandoned holds the terminal for its whole ceiling.

  @assistant
  Scenario: a pending answer reports how long it has been waiting
    Given inference is available
    And the inference provider endpoint accepts the connection and never answers
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the region titled "Ask Tinman" shows the elapsed seconds of the pending call
    And the reported elapsed seconds advance while the call is pending

  @assistant
  Scenario: escape abandons a pending answer and keeps the session
    Given inference is available
    And the inference provider endpoint accepts the connection and never answers
    And the operator runs "tinman --help" in an interactive terminal
    And the operator types "what does inspect do" at the assistant prompt
    When the operator presses "esc" at the assistant prompt
    Then the region titled "Ask Tinman" is drawn
    And the command has not exited

  Rule: the operator can correct what they typed. A prompt that only appends makes a typo unrecoverable, so an operator who mistypes must send the wrong question and ask again, which on this path costs a real model call. Text arrives as bytes and is shown as characters: a character outside ASCII is several bytes, and appending them one at a time renders one replacement mark per byte rather than the character the operator typed.

  @assistant
  Scenario: a typed character can be erased
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "recrd" at the assistant prompt without sending
    And the operator presses "backspace" at the assistant prompt
    Then the region titled "Ask Tinman" shows "recr"

  @assistant
  Scenario: a character outside ASCII is shown as the operator typed it
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "café" at the assistant prompt without sending
    Then the region titled "Ask Tinman" shows "café"

  Rule: declaring the interface in the terminal object model means the assistant is checked exactly as a test author checks their own program, and the model is exercised against a real screen on every run. Because the model carries presentation beside structure, one scantling serves as this interface's stylesheet and its markup at once, covering layout, measure, the cursor and the box's own styling. What a single screen cannot show stays in scenarios: behaviour over time, such as whether a reported wait advances, and the two colour renderings, since a contract naming one of them would be false under the other.

  @contract @assistant
  Scenario: the assistant interface conforms to its terminal object model contract
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal 120 columns wide
    Then the terminal object model of the screen conforms to the "assistant-ui" schema in "scantlings/assistant-ui.schema.json"

  @contract @assistant
  Scenario: the assistant interface conforms to its contract on a narrow terminal
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal 40 columns wide
    Then the terminal object model of the screen conforms to the "assistant-ui" schema in "scantlings/assistant-ui.schema.json"
    And the region titled "Ask Tinman" is at most 40 columns wide

  Rule: the border is drawn with rounded corners. A square-cornered box is the default every terminal program has drawn since curses, and the corner glyph is the whole difference between a box that looks considered and one that looks unstyled. It is asserted by the glyph rather than by eye, because a border style is exactly the kind of change a later refactor drops without any test noticing.

  @assistant
  Scenario: the box is drawn with rounded corners
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the region titled "Ask Tinman" has the corner glyph "╭"

  Rule: the key hints say what the keys do now, not what they usually do. While a call is pending, escape abandons that call rather than leaving the session, so a hint reading "esc to leave" would be advertising the wrong outcome at exactly the moment an operator reaches for it. Advertising a key that does something else is the same fault as advertising a command that does nothing.

  @assistant
  Scenario: the hint offers to cancel while an answer is pending
    Given inference is available
    And the inference provider endpoint accepts the connection and never answers
    And the operator runs "tinman --help" in an interactive terminal
    When the operator types "what does inspect do" at the assistant prompt
    Then the assistant prompt names "esc" as the key that cancels

  Rule: colour marks the box without carrying meaning, so an operator who cannot see it loses nothing. NO_COLOR is honoured because it is the convention every other terminal program already answers to, and a program that invents its own switch makes the operator configure it twice.

  @assistant
  Scenario: the box is drawn in colour
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the region titled "Ask Tinman" is drawn in a colour other than the default foreground

  @assistant
  Scenario: NO_COLOR draws the box without colour
    Given inference is available
    And the environment sets "NO_COLOR" to "1"
    When the operator runs "tinman --help" in an interactive terminal
    Then a bordered region titled "Ask Tinman" is drawn
    And no cell is drawn in a colour other than the default foreground

  Rule: the prompt names the keys that work it, because an operator dropped into a prompt has no other way to learn them and a terminal offers no menu to discover. The keys are asserted by name here rather than only through the asset body, since a scenario comparing output to an asset passes just as well when the asset loses the line.

  @assistant
  Scenario: the assistant prompt names the keys that send and leave
    Given inference is available
    When the operator runs "tinman --help" in an interactive terminal
    Then the assistant prompt names "enter" as the key that sends
    And the assistant prompt names "esc" as the key that leaves

  @assistant
  Scenario: the assistant session ends when the operator ends the input
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator ends the input
    Then the command exits with status 0

  @assistant
  Scenario: the assistant session ends when the operator presses escape
    Given inference is available
    And the operator runs "tinman --help" in an interactive terminal
    When the operator presses "esc" at the assistant prompt
    Then the command exits with status 0

  Rule: a reply reaches the propose-confirm path only by carrying the marker, and the instruction asset never named it, so every reply the model wrote fell through to prose and the whole path was unreachable in the shipped program. Two lists, one in code and one in copy, is the fault this project keeps paying for, and a reader is not what joins them.

  @assistant
  Scenario: the instruction asset teaches the marker the assistant reads
    Given the proposal marker the assistant reads
    When the asset at "assets/help/assistant-instruction.txt" is searched for it
    Then the instruction asset carries that marker

  Rule: the marker was read as a prefix of the whole reply, so a reply was entirely a proposal or entirely prose, and the model could not say what a command does and then offer it. Reading a marked line out of the reply leaves the security boundary where it was: whatever follows the marker still reaches the operating system only through Tinman's own parser.

  @assistant
  Scenario: a reply that answers and then proposes carries both
    Given the assistant replies with the answer "Record captures a live session." and the command "tinman record opencode"
    When the operator asks "how do I record opencode"
    Then the assistant displays the answer "Record captures a live session."
    And the assistant displays the proposed command "tinman record opencode"

  @assistant
  Scenario: a proposed command is displayed before it runs
    Given the assistant infers the command "tinman record opencode"
    When the operator asks "record the opencode agent"
    Then the assistant displays the proposed command "tinman record opencode"
    And the proposed command has not run

  @assistant
  Scenario: a declined proposal does not run
    Given the assistant has proposed the command "tinman record opencode"
    When the operator declines the proposal
    Then the proposed command has not run

  @assistant
  Scenario: a confirmed proposal runs through the command parser
    Given the assistant has proposed the command "tinman record opencode"
    When the operator confirms the proposal
    Then the command parser receives the arguments "record" and "opencode"

  @assistant
  Scenario: a proposal outside Tinman's command set is refused
    Given the assistant infers the command "rm -rf /"
    When the operator asks "delete everything"
    Then the assistant refuses the proposal
    And no command is offered to the operator

  @assistant
  Scenario: a proposal naming an unknown Tinman subcommand is refused
    Given the assistant infers the command "tinman teleport opencode"
    When the operator asks "teleport the opencode agent"
    Then the assistant refuses the proposal

  @assistant
  Scenario: a question is answered without proposing a command
    Given the assistant answers "Inspect prints the terminal object model of a running program."
    When the operator asks "what does inspect do"
    Then the assistant displays the answer "Inspect prints the terminal object model of a running program."
    And no command is offered to the operator

  @contract @assistant
  Scenario: the assistant discharges the command-parser boundary contract
    Given the interactive assistant source
    When the verifier checks the assistant command boundary
    Then no counterexample is found

  Rule: the assistant box reads its title and its key hints from assets/help/assistant-prompt.txt, and the setup form beside it held the same kinds of copy as constants in the setup implementation. One screen carries two catalogues, and only one of them is a catalogue an operator or a translator can edit without touching the implementation. assets/help/setup-form.txt was written to close that, and the scenario below then compared the form against the asset and passed while the form still drew from the constants, because those constant values had been copied out of the asset by hand. Equal strings are not a reading, and the scenario beneath this one is what tells the two apart.

  Rule: the label on the credential field is the one piece of that copy the asset does not carry, and the reason is arithmetic rather than oversight. Its value ends in a space the field's width calculation counts, so a line-oriented text asset would make a trailing space the difference between a correct form and a misaligned one, invisible in every editor that trims on save. A catalogue an operator can edit is what the move was for, and a value whose end they cannot see is not one.

  @assistant
  Scenario: the setup form draws the title and key hints the assets carry
    Given the operator has opened the setup form
    Then the form title is the title the setup asset carries
    And the form names the keys that asset carries as the keys that save and leave
    And the form names the environment variable that asset carries

  Rule: a catalogue an operator can edit is what the move to assets was for, and the scenario above cannot tell a form that reads its asset from a form that carries a copy of it. Both satisfy an assertion comparing the drawn line against the asset's line, because the two strings agree until one of them is edited. Editing the asset and finding the screen followed would tell them apart, and no run can do it: the assets are compiled into the binary, so the edit a test would make is one only a rebuild can apply. The reachable fact is the structural one underneath, that the implementation holds no second copy to drift from, and the contract below is what holds it.

  @contract @assistant
  Scenario: the operator-facing copy is drawn from the asset catalogue
    Given the implementation sources
    When the verifier checks the copy catalogue boundary
    Then no counterexample is found
