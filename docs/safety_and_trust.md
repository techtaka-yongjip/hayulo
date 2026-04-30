# Safety and Trust

Hayulo is designed for AI-assisted software creation. That makes safety central. Generated code can be useful, but it can also make mistakes quickly and confidently.

A language for this workflow must help people see, restrict, test, and review what generated code can do.

## Safety goals

Hayulo should help answer:

- What can this program access?
- What can this program change?
- What data does it process?
- What external systems does it call?
- What actions require human approval?
- What tests cover important behavior?
- What changed between versions?
- Did an AI-generated patch expand risk?

## Safety is not only runtime sandboxing

Sandboxing matters, but it is not enough.

Hayulo safety should include:

- language design
- type checking
- `Option` and `Result`
- tests
- effects
- permissions
- policy checks
- package integrity
- audit logs
- human review
- good defaults

## Effects

Effects describe what code can do beyond pure computation.

Examples:

```text
files.read
files.write
files.delete
network.read
network.write
db.read
db.write
env.read
clock.read
email.send
money.spend
user_visible_action
irreversible_action
sensitive_data.process
ai.call
```

Future Hayulo code could declare effects:

```hayulo
fn export_report(path: Path, report: Report) -> Result<(), FileError>
  effects [files.write]
{
  return files.write_text(path, report.to_text())
}
```

The compiler can then compare required effects against project policy.

## Permissions

Project permissions should live in `hayulo.toml`:

```toml
[permissions]
allow = [
  "files.read",
  "files.write",
  "network.read"
]

deny = [
  "files.delete",
  "money.spend"
]

require_approval = [
  "email.send",
  "irreversible_action"
]
```

Generated code should not be allowed to silently change permissions without review.

## Approval gates

Some actions should require explicit approval:

- sending messages
- making purchases
- deleting records
- modifying production data
- sharing sensitive information
- calling external APIs with private data
- executing generated shell commands

Future Hayulo syntax might include:

```hayulo
approval = ask approval user {
  action: "Send invoice email"
  preview: email.body
  effects: [email.send]
}

if approval.granted {
  email.send(email)?
}
```

## Sensitive data

Hayulo should help mark sensitive data:

```hayulo
type PatientRecord = record sensitive {
  name: Text
  date_of_birth: Date
  notes: Text
}
```

Future checkers could warn when sensitive data is logged, sent over the network, or passed to an AI model without policy approval.

## AI model calls

If Hayulo includes AI APIs, model calls should be explicit effects:

```text
ai.call
network.write
sensitive_data.process maybe
```

Model outputs should carry uncertainty when appropriate. For example:

```hayulo
Belief<InvoiceData>
```

instead of raw `InvoiceData`.

This makes it clear that extraction or classification may need verification.

## Dependency trust

Package ecosystems create supply-chain risk.

Hayulo package management should eventually support:

- lockfiles
- checksums
- package signing
- permission declarations
- dependency effect summaries
- security advisories
- package audit command

Command idea:

```bash
hayulo audit
```

Output should include whether dependencies request risky effects.

## Safe defaults

Hayulo should default to:

- no file delete permission
- no money spending
- no email sending
- no external network access without declaration
- no hidden dependency execution
- no null values
- no unchecked missing values
- no ignored recoverable errors

Users can opt into power, but the default should be conservative.

## Runtime sandboxing

Long-term, Hayulo should support sandboxed execution.

Possible approaches:

- WebAssembly runtime
- OS-level sandbox
- container execution
- restricted interpreter mode
- permissioned standard library APIs

The sandbox should enforce what the compiler checks statically.

## Auditing and logs

High-impact actions should produce audit logs:

- when the action happened
- what code path triggered it
- what permissions were used
- whether approval was granted
- what data was involved, with redaction where appropriate

Audit logs should be designed with privacy in mind.

## Threat model

Hayulo should eventually publish a formal threat model for:

- untrusted Hayulo code
- malicious packages
- prompt-injected code generation
- AI-generated insecure patches
- dependency confusion
- sensitive data leakage
- unsafe tool execution

## Current prototype warning

The current seed interpreter is experimental and not production-ready. It should not be used to run untrusted code in sensitive environments.

The safety model described here is the intended direction, not the current implementation.
