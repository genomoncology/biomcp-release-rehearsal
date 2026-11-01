# 0526 — Check the module-shape rules

Status: COMPLETE.
Landed: b5c7096bb34a31180c3142defcdbf8cab2d36179

Milestone: 5

## Outcome

A structure ratchet holds two Rust rules that no tool checks today: parent modules hold declarations rather than logic, and every source file starts with a module doc. Each count can only fall.

- Why: Jarvis's 1.0 code review (m17823, finding 10). 134 of 178 parent modules hold logic, and 414 of 1,041 source files lack a first-line module doc. The workspace rule asks for useful conventions to be enforced by checks.
- Starts from: the existing structure ratchets and the Rust rules file.
- Rules: the ratchet records today's counts and fails when either count rises. Files are not moved in this ticket.
- Proof: the ratchet fails when a new file without a module doc is added on purpose, or when logic is added to a clean parent module, and passes on main.
- Changes: the ratchets.
- Defers: driving either count to zero.
