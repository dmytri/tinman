Feature: terminal object model inference
  As a test author recording a session
  I want inference to read the screen and name what it sees
  So that the generated plan addresses roles and names instead of coordinates

  Rule: inference enriches the terminal object model at capture time only. The deterministic model is the spine; the inference engine is a second producer of the same shape, so a plan authored by hand needs no model at all.

  Rule: replay rebuilds the model with no model invocation, so it can only find a name the screen still yields. That asymmetry is why an inferred name cannot be taken on the engine's word: a name the model invented reads exactly like a name the screen carries, right up until something tries to bind it, and by then the operator is gone.

  Rule: a mis-bound action announces itself on the next step, so forgiveness there is recoverable. An assertion is the thing that would have caught it, so the same forgiveness destroys it silently: an assertion that rebinds to a region it did not name passes while asserting nothing, and an absence assertion that rebinds fails while the product is correct.

  Rule: resolving a locator and confirming one are two operations, not one. Resolution answers what a locator matches in the model as it stands, and reports ambiguity as ambiguity, which is what a replaying test needs. Confirmation runs at capture time only: it takes a proposed locator, and where resolution reports ambiguity it narrows by scope or ordinal until exactly one region binds, recording which of `exact`, `scoped` or `ordinal` it needed. Collapsing them would make an ambiguous locator look bindable to the test that must later resolve it alone.

  Rule: inference proposes names, never roles. Every role in the model schema is derived deterministically, because a plan may address any of them and replay rebuilds the model with no model invocation. What the deterministic pass cannot supply is a name for a region carrying no title, and a name it does propose must still be text the screen actually shows.

  Scenario: an expectation whose locator needs a fallback is refused
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And an engine that names the first item "source directory"
    When an expectation on that item is recorded
    Then recording fails and reports the expectation's locator did not bind

  Scenario: a confirmed locator is written as inference proposed it
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And an engine that names the pane "Files"
    When the inferred locator is round-tripped against the deterministic model
    Then the locator binds to the region named "Files"

  Scenario: a name the screen does not carry is rejected
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And an engine that names the pane "File Browser"
    When the inferred locator is round-tripped against the deterministic model
    Then the locator is rejected as unbindable

  Scenario: a name matching two regions is scoped to its parent
    Given a virtual screen showing two bordered panes each listing an item named "README"
    And an engine that names the second item "README"
    When the inferred locator is round-tripped against the deterministic model
    Then the locator is scoped to the region containing that item

  Scenario: a rejected name falls back to a deterministic locator
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And an engine that names the first item "source directory"
    When the inferred locator is round-tripped against the deterministic model
    Then the locator addresses the first "listitem" of the region named "Files"

  Scenario: the written plan names the fallback a locator needed
    Given a virtual screen showing two bordered panes each listing an item named "README"
    And an engine that names the second item "README"
    When the plan is written
    Then the plan records the locator's binding as "scoped"

  Scenario: an unavailable engine leaves the deterministic model standing
    Given inference is unavailable
    And a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    When the terminal object model is inferred
    Then the region named "Files" has the role "list"

  Scenario: inference names a region the deterministic pass left unnamed
    Given a virtual screen showing an unbordered pane whose first line reads "Recent files"
    And an engine that names that region "Recent files"
    When the terminal object model is inferred
    Then the model contains a region named "Recent files"

  Scenario: an engine result that violates the model schema is discarded
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And an engine that returns a region with the role "wormhole"
    When the terminal object model is inferred
    Then the region named "Files" has the role "list"

  Rule: text that parses as JSON is not thereby a model. A provider answering `null`, a bare string or an array has returned nothing a model can be read from, so the deterministic model stands exactly as it does when no credential is configured and when the provider cannot be reached. A real provider does answer this way: an observed sweep failed on the four characters `null` against production that had passed twice, which is the same reply arriving on a different day.

  Scenario: an engine answering with a value that is not a model is discarded
    Given a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And an engine that answers "null"
    When the terminal object model is inferred
    Then the region named "Files" has the role "list"

  Scenario: a region the engine leaves unnamed keeps its deterministic reading
    Given a virtual screen showing an unbordered pane whose first line reads "Recent files"
    And an engine that answers with a region carrying no name
    When the terminal object model is inferred
    Then that region carries the name the deterministic reading gave it

  Rule: tldr pages describe what a command is for in the words its own community chose, and that is the vocabulary a good name comes from. The engine is naming regions of a program it has never seen, so a page describing that program is worth carrying. A page also describes some version of a program, and the drift between it and the binary on this machine is the drift that made Tinman's own help text describe a parser it had diverged from; carried into naming, a stale page costs a worse suggestion, where taken as the basis for an assertion it would write a claim about a program nobody ran. The network objection was Captain's error and the user caught it: network is denied to the target, never to Tinman, which already calls a provider from outside the sandbox.

  Rule: the tldr-pages project keeps a public list of projects that take its pages, run them through a language model and publish the result without crediting it, describing that output as inaccurate and riddled with hallucinations. Feeding a page to an inference engine and emitting names from it is that shape exactly, so what keeps Tinman off the list has to be structural rather than well meant. Two things do. The pages are licensed CC-BY-4.0, so credit is a licence term and not a courtesy, and it belongs in the artifact the operator commits rather than on a screen they saw once. And no text from a page reaches a Tinman artifact on the page's authority alone: the deterministic pass refuses any name the screen does not independently carry, which is the refusal every other inferred name already meets.

  Rule: the page source is configuration, the way the inference endpoint already is. It defaults to the tldr-pages project's own raw markdown and is pointed elsewhere by an environment variable, which serves an operator behind a mirror or working offline, and lets this project's own verification fetch a real page over real HTTP from a source it controls rather than depending on what the public project happens to serve today. A page whose text the project may edit at any time is not a fixture, and faking the fetch instead is the doubling the Real-by-default Article forbids.

  Scenario: the page source is the tldr project unless configured otherwise
    Given no tldr page source is configured
    When the page source is read
    Then it is the tldr-pages project's raw markdown

  Scenario: a configured page source is read instead
    Given the environment sets "TINMAN_TLDR_BASE_URL" to "https://mirror.example.com/tldr"
    When the page source is read
    Then it is "https://mirror.example.com/tldr"

  Rule: Tinman is not a tldr client and does not intend to become one. The client specification carries required flags, platform resolution, language handling and cache maintenance, all of it off Tinman's mission, and a second copy of somebody else's pages is a second thing to serve stale. Declining the specification is not declining a request: one page is fetched as raw markdown from the project, which needs no client installed and no cache kept. Markdown is the form the style guide specifies, so placeholders and keypress notation arrive as written rather than as somebody's rendering of them. Platform is resolved by asking for the platform page and falling back to the common one, which is two attempts rather than a cache. The page is under CC-BY-4.0 and the credit rides in the plan.

  Scenario: the naming context carries a tldr page for the program being inferred
    Given the configured tldr page source has a page for "git"
    And a virtual screen showing an unbordered pane whose first line reads "Recent files"
    When the terminal object model of "git" is inferred
    Then the page is read as raw markdown from the tldr project
    And the inference request carries the tldr page for "git"

  Scenario: inference proceeds where no tldr page is available
    Given the configured tldr page source has no page for "obscurecmd"
    And a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    When the terminal object model of "obscurecmd" is inferred
    Then the inference request carries no tldr page
    And the region named "Files" has the role "list"

  Scenario: a name only the tldr page carries is rejected like any other
    Given the configured tldr page source has a page for "git"
    And a virtual screen showing a bordered pane titled "Files" listing "src" and "tests"
    And an engine that names the pane "repository browser"
    When the inferred locator is round-tripped against the deterministic model
    Then the locator is rejected as unbindable

  Scenario: a plan whose naming read a tldr page credits the project
    Given the configured tldr page source has a page for "git"
    And a virtual screen showing an unbordered pane whose first line reads "Recent files"
    And an engine that names that region "Recent files"
    When the plan for "git" is written
    Then the plan credits the tldr-pages project for the page it read
    And the plan names "CC-BY-4.0" as that page's licence

  Scenario: a plan whose naming read no tldr page credits nothing
    Given the configured tldr page source has no page for "obscurecmd"
    And a virtual screen showing an unbordered pane whose first line reads "Recent files"
    And an engine that names that region "Recent files"
    When the plan for "obscurecmd" is written
    Then the plan credits no page

  Rule: an @inference scenario asserts Tinman's seam and never the provider's latency. The seam is the request Tinman builds, the call it makes and the reply it parses; how long a third party takes to answer is that party's behaviour, the same class as whether it obeys an instruction. A single real call returning nothing inside its ceiling is a transient of a hosted service, so the step retries toward a deadline rather than failing on one slow answer. The cost of getting this wrong is measured rather than argued: with production byte-identical across two custody attempts, the tier sweep moved from 85s green to 159s red, and a ceiling sized off a 76s observed tail was exceeded within a day of being set. Chasing that tail with a larger constant buys the next reprieve and no more.

  @inference
  Rule: what conforms is the model Tinman produces, never the reply a provider sent. Asserting the reply would be asserting the model's compliance, which this tier does not do and cannot enforce; shaping the request is not validating the response. The neighbouring scenarios already discard a reply that is not a model at all, and a reply that is a model with a region missing something the schema requires is the same case arriving in part rather than whole: the enrichment is refused for that region and the deterministic reading stands there. A real engine returns exactly that, a null name on a role whose name is required, so this is the ordinary case rather than an edge.

  Scenario: the model Tinman produces from a real screen conforms
    Given the fixture terminal program is captured through a PTY
    When the terminal object model is inferred by the configured engine
    Then the model Tinman produces conforms to the "tom" schema in "scantlings/tom.schema.json"
