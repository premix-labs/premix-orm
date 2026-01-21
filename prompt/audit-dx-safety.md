<!-- audit-dx-safety.md -->

Role: Senior Rust Developer Experience (DX) Specialist & Security Auditor.

Objective: Evaluate "Premix ORM" to ensure it adheres to the "Pit of Success" philosophy—making it easy to do the right thing and impossible to do the wrong thing. Focus on API ergonomics, compile-time safety rails, and frictionless adoption.

Instructions for Agent:

1. 🧠 Mental Model & Ergonomics Analysis
   The "Fluent" Check: Analyze the chaining of methods (e.g., User::find().filter(...).include(...)).

Goal: Does it read like English? Are the transition states logical?

IDE Support: CRITICAL: Does the macro-generated code play nicely with rust-analyzer? Do users get autocomplete hints for fields and methods, or is it a "black box"?

Boilerplate vs. Clarity:

Audit the #[derive(Model)] requirements. Do users need to import 10 traits to make it work, or is a single prelude (use premix::prelude::\*;) sufficient?

2. 🛡️ The "Impossible-to-Fail" Safety Audit
   Destructive Guards (Type State Pattern):

Attempt to write User::delete_all().await.

Verification: This MUST NOT compile. The builder should require a state transition via .filter(...) or .allow_unsafe() to transform into an executable future.

Runtime Check: If compile-time checks are bypassed (e.g., via unsafe), does the runtime throw a panic or error?

Type-Safety Leakage:

Try to pass a String into an integer field query (e.g., user.id.eq("some-string")).

Goal: Verify that the Type System catches this immediately. Ensure strict typing matches the SQL column types.

3. 🚨 Compiler Diagnostics & Error Messages
   The "Typo" Test: Introduce a typo in a field name inside a macro (e.g., #[premix(colum_name = "...")]).

Check: Does the error message point exactly to the typo line (using syn::spanned)?

Bad DX: Error points to the #[derive(Model)] macro at the top of the struct (User has no idea where the error is).

Good DX: Error points to line 15, column 4: "Unknown attribute 'colum_name', did you mean 'column_name'?"

4. 🕵️ Security & Privacy (Glass Box)
   Sensitive Data Masking:

Create a struct with #[premix(sensitive)] on a password field.

Run println!("{:?}", user); and check the logs.

Pass Condition: The log must show password: "\*\*\*".

Fail Condition: The raw password is visible in logs or SQL trace outputs.

5. 🐣 The "Newbie" Simulation (The Blog Test)
   Scenario: Attempt to implement a basic "Blog System" (User has_many Posts) using only the provided README/Docs.

Friction Log: Record every time you have to "guess" how to do something because the API isn't self-explanatory.

Async/Await Pitfalls: Are there confusing moments where await is missing or placed incorrectly due to API design?

Reporting Format:

Friction Points: Top 3 API interactions that felt clunky or required checking the source code.

Safety Breach Report: Can delete_all be called without safeguards? (Yes/No).

IDE Compatibility: Did autocomplete work inside the query builder?

Refactoring Advice: Specific suggestions to improve method naming or trait organization.
