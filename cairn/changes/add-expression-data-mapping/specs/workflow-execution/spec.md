## ADDED Requirements

### Requirement: Expression syntax
r[n8nclone.workflow_execution.expression_mapping.syntax] The system MUST define a deterministic expression syntax for referencing workflow data.

#### Scenario: Parse output reference
r[n8nclone.workflow_execution.expression_mapping.syntax.reference]
- GIVEN an expression references an upstream node output by stable node and port identity
- WHEN the expression parser runs
- THEN the system produces an expression AST that identifies the referenced value

#### Scenario: Reject invalid syntax
r[n8nclone.workflow_execution.expression_mapping.syntax.invalid]
- GIVEN an expression contains invalid syntax
- WHEN workflow validation runs
- THEN the system reports a stable expression diagnostic before execution starts

### Requirement: Expression evaluation
r[n8nclone.workflow_execution.expression_mapping.evaluate] The system MUST evaluate expressions against completed upstream workflow values before executing dependent nodes.

#### Scenario: Interpolate upstream stdout
r[n8nclone.workflow_execution.expression_mapping.evaluate.stdout]
- GIVEN an expression interpolates an upstream shell node's stdout
- WHEN the upstream node completes and the dependent node is prepared
- THEN the system evaluates the expression to the upstream stdout value

#### Scenario: Evaluate bounded transform
r[n8nclone.workflow_execution.expression_mapping.evaluate.transform]
- GIVEN an expression applies a supported bounded transform to an upstream value
- WHEN expression evaluation runs
- THEN the system returns the transformed value without performing I/O or process execution

### Requirement: Expression-backed node configuration
r[n8nclone.workflow_execution.expression_mapping.config] The system SHOULD allow expression values in supported node configuration fields.

#### Scenario: Resolve command argument
r[n8nclone.workflow_execution.expression_mapping.config.command]
- GIVEN a shell node command field contains an expression
- WHEN the node is prepared after its dependencies complete
- THEN the system resolves the expression before invoking the shell adapter

#### Scenario: Resolve HTTP body
r[n8nclone.workflow_execution.expression_mapping.config.http_body]
- GIVEN an HTTP node body field contains an expression
- WHEN the node is prepared after its dependencies complete
- THEN the system resolves the expression before invoking the HTTP adapter

### Requirement: Expression error handling
r[n8nclone.workflow_execution.expression_mapping.errors] The system MUST fail dependent node preparation when expression references are missing, mistyped, or unavailable.

#### Scenario: Missing reference
r[n8nclone.workflow_execution.expression_mapping.errors.missing]
- GIVEN an expression references a node or port that is not available in the evaluation context
- WHEN the dependent node is prepared
- THEN the system fails that node before executing any side-effecting adapter

#### Scenario: Type mismatch
r[n8nclone.workflow_execution.expression_mapping.errors.type]
- GIVEN an expression result cannot be converted to the target field type
- WHEN the dependent node is prepared
- THEN the system reports a type diagnostic and does not execute the node

### Requirement: Expression redaction
r[n8nclone.workflow_execution.expression_mapping.redaction] The system MUST preserve sensitivity metadata when expressions derive values from secrets.

#### Scenario: Secret-derived expression output
r[n8nclone.workflow_execution.expression_mapping.redaction.secret]
- GIVEN an expression combines a secret-derived value with non-secret text
- WHEN the expression result is rendered in logs, TUI, history, or JSON summaries
- THEN the system redacts the secret-derived portion according to shared redaction rules
