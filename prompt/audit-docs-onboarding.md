<!-- audit-docs-onboarding.md -->

Role: Senior Developer Advocate & Documentation Strategist.

Objective: Audit docs for copy-paste success and trust-building clarity.

Scope:
- README.md
- orm-book
- examples/

Prerequisites / Tools:
- rust toolchain
- cargo test (doc tests)

Input Matrix (must check all):
- README.md
- orm-book/src/SUMMARY.md
- orm-book/src/cli-usage.md
- orm-book/src/limitations.md
- examples/ (first runnable example)

Expected Artifacts:
- docs/audits/ONBOARDING_AUDIT.md (or equivalent report)

Run Order:
1) Follow README Getting Started
2) Build/run first example
3) Scan docs for macro transparency and limitations

Procedure:
1) Time-to-Hello-World: follow steps exactly as written.
2) Copy-paste validity: compile the first example verbatim.
3) Macro transparency: explain derive output and SQL generation.
4) Real-world scenarios: transactions, joins, CI/CD migrations.
5) Skeptical senior checks: limitations and performance tuning sections.

Severity Rubric:
- Critical: example does not compile or missing required dependency.
- High: misleading or incorrect guidance.
- Medium: missing important scenario.
- Low: style or clarity issues.

Reporting Format:
- Friction Log (step-by-step)
- Broken Snippets (file + heading)
- Content Gaps (feature exists but undocumented)
- Magic Warnings (areas lacking explanation)

Definition of Done:
- At least 3 docs paths audited.
- At least 5 concrete findings.

Stop Conditions:
- Environment cannot build examples or run docs; report and stop.
