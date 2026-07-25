Feature: inspect command
  As a test author
  I want to see the terminal object model of a running program
  So that I can discover the roles and names my test should address

  Scenario: inspect lists the roles and names on the current screen
    When the operator inspects the fixture terminal program
    Then the inspect output lists a "menuitem" named "Settings"

  Scenario: inspect emits the model as JSON when asked
    When the operator inspects the fixture terminal program as JSON
    Then the inspect output conforms to the "tom" schema in "scantlings/tom.schema.json"

  Scenario: inspect reports a program that draws nothing
    When the operator inspects the command "true"
    Then the inspect output reports "no regions on screen"
