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

  Rule: an @inference scenario asserts Tinman's seam and never the provider's latency. The seam is the request Tinman builds, the call it makes and the reply it parses; how long a third party takes to answer is that party's behaviour, the same class as whether it obeys an instruction. A single real call returning nothing inside its ceiling is a transient of a hosted service, so the step retries toward a deadline rather than failing on one slow answer. The cost of getting this wrong is measured rather than argued: with production byte-identical across two custody attempts, the tier sweep moved from 85s green to 159s red, and a ceiling sized off a 76s observed tail was exceeded within a day of being set. Chasing that tail with a larger constant buys the next reprieve and no more.

  @inference
  Scenario: the configured engine infers a conforming model from a real screen
    Given the fixture terminal program is captured through a PTY
    When the terminal object model is inferred by the configured engine
    Then it conforms to the "tom" schema in "scantlings/tom.schema.json"

  @inference
  Scenario: the configured engine generates an acronym expansion
    When an acronym expansion is generated by the configured engine
    Then a non-empty expansion is produced
