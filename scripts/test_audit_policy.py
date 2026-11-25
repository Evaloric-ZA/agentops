#!/usr/bin/env python3
"""The regression tests for the two dependency audit gates.

A gate that has never been shown to fail is not a gate. Every rule the
gates enforce is checked here with findings that no registry and no
advisory database would ever produce, so the tests do not depend on the
network and do not depend on what the advisory databases happen to say
today.

The tests cover:

- the allowlist schema: a missing field, an empty field, a malformed
  date, a duplicate entry, an unknown scope, a review date that is not
  after the date the entry was added, a missing file and a file that
  is not JSON are all problems, and a problem fails the gate rather
  than being quietly ignored;
- the expiry rule: an exception past its review date has expired, and
  nothing renews it;
- the npm policy: Critical and High fail, Moderate warns, Low and
  informational are reported, an accepted advisory passes, an expired
  exception fails the release gate, and an unknown advisory is never
  accepted;
- the Rust policy: a vulnerability and an informational advisory both
  fail unless they are accepted, and the same expiry and staleness
  rules apply;
- the repository's own two allowlists: they parse, they satisfy the
  schema, and every entry says what would close it.

Standard library only. It makes no network request, writes nothing
outside a temporary directory and deletes nothing.
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from datetime import date
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from audit_policy import (  # noqa: E402
    NPM_REQUIRED,
    NPM_SCOPES,
    RUST_REQUIRED,
    RUST_SCOPES,
    expired,
    load_allowlist,
)
from npm_audit_gate import decide as npm_decide  # noqa: E402
from npm_audit_gate import findings as npm_findings  # noqa: E402
from rust_audit_gate import decide as rust_decide  # noqa: E402
from rust_audit_gate import found as rust_found  # noqa: E402

TODAY = date(2026, 11, 25)


def an_entry(**overrides) -> dict:
    entry = {
        "advisory": "GHSA-xxxx-yyyy-zzzz",
        "package": "example",
        "reason": "it is only used by the test toolchain",
        "scope": "development",
        "added": "2026-11-01",
        "review_by": "2026-12-01",
        "remediation": "upgrade the parent when a release fixes it",
        "owner": "project-maintainers",
    }
    entry.update(overrides)
    return entry


def a_rust_entry(**overrides) -> dict:
    entry = {
        "advisory": "RUSTSEC-2026-0000",
        "crate": "example",
        "reason": "it is not reachable from the application",
        "scope": "runtime",
        "added": "2026-11-01",
        "review_by": "2026-12-01",
        "remediation": "upgrade the upstream dependency",
        "upstream_dependency": "example/example",
        "status": "temporarily-accepted",
    }
    entry.update(overrides)
    return entry


class TheAllowlistSchema(unittest.TestCase):
    """The schema the two allowlists have to satisfy."""

    def write(self, data: object) -> Path:
        directory = Path(tempfile.mkdtemp(prefix="agentops-policy-"))
        path = directory / "allowlist.json"
        if isinstance(data, str):
            path.write_text(data, encoding="utf-8")
        else:
            path.write_text(json.dumps(data), encoding="utf-8")
        return path

    def problems_for(self, data: object) -> list[str]:
        path = self.write(data)
        _, problems = load_allowlist(path, NPM_REQUIRED, "the test allowlist", NPM_SCOPES)
        return problems

    def test_a_complete_entry_has_no_problems(self) -> None:
        path = self.write({"advisories": [an_entry()]})
        _, problems = load_allowlist(path, NPM_REQUIRED, "the test allowlist", NPM_SCOPES)
        self.assertEqual(problems, [])

    def test_an_empty_list_has_no_problems(self) -> None:
        path = self.write({"advisories": []})
        _, problems = load_allowlist(path, NPM_REQUIRED, "the test allowlist", NPM_SCOPES)
        self.assertEqual(problems, [])

    def test_a_missing_file_is_a_problem(self) -> None:
        missing = Path(tempfile.mkdtemp(prefix="agentops-policy-")) / "nothing.json"
        entries, problems = load_allowlist(
            missing, NPM_REQUIRED, "the test allowlist", NPM_SCOPES
        )
        self.assertEqual(entries, [])
        self.assertTrue(problems)

    def test_a_file_that_is_not_json_is_a_problem(self) -> None:
        self.assertTrue(self.problems_for("{not json"))

    def test_a_file_that_is_not_an_object_is_a_problem(self) -> None:
        self.assertTrue(self.problems_for("[1, 2, 3]"))

    def test_a_missing_field_is_a_problem(self) -> None:
        entry = an_entry()
        del entry["review_by"]
        self.assertTrue(self.problems_for({"advisories": [entry]}))

    def test_an_empty_field_is_a_problem(self) -> None:
        self.assertTrue(self.problems_for({"advisories": [an_entry(reason="  ")]}))

    def test_a_malformed_date_is_a_problem(self) -> None:
        self.assertTrue(self.problems_for({"advisories": [an_entry(added="01-11-2026")]}))

    def test_a_duplicate_advisory_is_a_problem(self) -> None:
        problems = self.problems_for({"advisories": [an_entry(), an_entry()]})
        self.assertTrue(any("more than once" in problem for problem in problems))

    def test_an_unknown_scope_is_a_problem(self) -> None:
        self.assertTrue(self.problems_for({"advisories": [an_entry(scope="sometimes")]}))

    def test_a_review_date_before_the_added_date_is_a_problem(self) -> None:
        problems = self.problems_for(
            {"advisories": [an_entry(added="2026-11-01", review_by="2026-10-01")]}
        )
        self.assertTrue(any("not after" in problem for problem in problems))

    def test_a_review_date_on_the_added_date_is_a_problem(self) -> None:
        problems = self.problems_for(
            {"advisories": [an_entry(added="2026-11-01", review_by="2026-11-01")]}
        )
        self.assertTrue(problems)


class TheExpiryRule(unittest.TestCase):
    """An exception is a dated decision, and the date is enforced."""

    def test_an_exception_in_the_future_has_not_expired(self) -> None:
        self.assertFalse(expired(an_entry(review_by="2026-12-01"), TODAY))

    def test_an_exception_on_the_day_is_not_yet_expired(self) -> None:
        self.assertFalse(expired(an_entry(review_by="2026-11-25"), TODAY))

    def test_an_exception_the_day_before_has_expired(self) -> None:
        self.assertTrue(expired(an_entry(review_by="2026-11-24"), TODAY))

    def test_an_exception_with_no_date_has_expired(self) -> None:
        self.assertTrue(expired(an_entry(review_by=""), TODAY))

    def test_an_exception_with_a_malformed_date_has_expired(self) -> None:
        self.assertTrue(expired(an_entry(review_by="soon"), TODAY))


class TheNpmPolicy(unittest.TestCase):
    """The severity policy and the exception rules for the npm audit."""

    def row(self, severity: str, advisory: str = "GHSA-aaaa-bbbb-cccc") -> dict:
        return {
            "package": "example",
            "advisory": advisory,
            "title": "an advisory",
            "severity": severity,
            "direct": False,
            "range": "<=1.0.0",
            "fix": None,
        }

    def test_an_unknown_critical_fails(self) -> None:
        _, failures = npm_decide([self.row("critical")], {}, release=True)
        self.assertTrue(failures)

    def test_an_unknown_high_fails(self) -> None:
        _, failures = npm_decide([self.row("high")], {}, release=True)
        self.assertTrue(failures)

    def test_an_unknown_moderate_does_not_fail(self) -> None:
        outcomes, failures = npm_decide([self.row("moderate")], {}, release=True)
        self.assertEqual(failures, [])
        self.assertEqual(outcomes[0][0], "WARN")

    def test_an_unknown_low_does_not_fail(self) -> None:
        outcomes, failures = npm_decide([self.row("low")], {}, release=True)
        self.assertEqual(failures, [])
        self.assertEqual(outcomes[0][0], "REPORT")

    def test_an_accepted_advisory_passes(self) -> None:
        allowed = {"GHSA-aaaa-bbbb-cccc": an_entry(review_by="2026-12-01")}
        outcomes, failures = npm_decide([self.row("critical")], allowed, release=True)
        self.assertEqual(failures, [])
        self.assertEqual(outcomes[0][0], "NOTE")

    def test_an_expired_exception_fails_the_release_gate(self) -> None:
        allowed = {"GHSA-aaaa-bbbb-cccc": an_entry(review_by="2026-11-01")}
        _, failures = npm_decide([self.row("critical")], allowed, release=True)
        self.assertTrue(failures)

    def test_an_expired_exception_only_warns_for_a_developer(self) -> None:
        allowed = {"GHSA-aaaa-bbbb-cccc": an_entry(review_by="2026-11-01")}
        outcomes, failures = npm_decide([self.row("critical")], allowed, release=False)
        self.assertEqual(failures, [])
        self.assertEqual(outcomes[0][0], "WARN")

    def test_an_exception_for_an_advisory_that_is_gone_fails_the_release_gate(self) -> None:
        allowed = {"GHSA-aaaa-bbbb-cccc": an_entry(review_by="2026-12-01")}
        _, failures = npm_decide([], allowed, release=True)
        self.assertTrue(failures)

    def test_an_exception_for_an_advisory_that_is_gone_only_warns_for_a_developer(self) -> None:
        allowed = {"GHSA-aaaa-bbbb-cccc": an_entry(review_by="2026-12-01")}
        outcomes, failures = npm_decide([], allowed, release=False)
        self.assertEqual(failures, [])
        self.assertTrue(any("did not find it" in message for _, message in outcomes))


class TheNpmFindings(unittest.TestCase):
    """Reading what npm found, and how it is classified."""

    def test_a_direct_advisory_is_read(self) -> None:
        data = {
            "metadata": {"vulnerabilities": {"critical": 1, "total": 1}},
            "vulnerabilities": {
                "tinypool": {
                    "severity": "critical",
                    "isDirect": False,
                    "range": "<=2.1.1",
                    "via": [
                        {
                            "title": "Prototype Pollution gadget",
                            "url": "https://github.com/advisories/GHSA-5gmw-xhrv-c9v3",
                            "severity": "critical",
                            "range": "<=2.1.0",
                        }
                    ],
                }
            },
        }
        rows = npm_findings(data)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["advisory"], "GHSA-5gmw-xhrv-c9v3")
        self.assertEqual(rows[0]["severity"], "critical")
        self.assertEqual(rows[0]["package"], "tinypool")

    def test_a_package_that_only_depends_on_a_vulnerable_one_is_not_a_finding(self) -> None:
        data = {
            "vulnerabilities": {
                "vitest": {
                    "severity": "critical",
                    "isDirect": True,
                    "via": ["tinypool"],
                }
            }
        }
        self.assertEqual(npm_findings(data), [])

    def test_the_rolled_up_severity_is_not_lowered(self) -> None:
        data = {
            "vulnerabilities": {
                "vite": {
                    "severity": "high",
                    "isDirect": False,
                    "via": [
                        {
                            "title": "an advisory",
                            "url": "https://github.com/advisories/GHSA-aaaa-bbbb-cccc",
                            "severity": "moderate",
                        }
                    ],
                }
            }
        }
        rows = npm_findings(data)
        self.assertEqual(rows[0]["severity"], "high")


class TheRustPolicy(unittest.TestCase):
    """The policy for the Rust advisories."""

    def row(self, kind: str = "vulnerability", advisory: str = "RUSTSEC-2026-0000") -> dict:
        return {
            "id": advisory,
            "package": "example",
            "version": "1.0.0",
            "kind": kind,
            "title": "an advisory",
            "patched": [],
        }

    def test_an_unaccepted_vulnerability_fails(self) -> None:
        _, failures = rust_decide([self.row()], {}, release=True)
        self.assertTrue(failures)

    def test_an_unaccepted_informational_advisory_fails(self) -> None:
        _, failures = rust_decide([self.row("unmaintained")], {}, release=True)
        self.assertTrue(failures)

    def test_an_accepted_advisory_passes(self) -> None:
        allowed = {"RUSTSEC-2026-0000": a_rust_entry(review_by="2026-12-01")}
        outcomes, failures = rust_decide([self.row()], allowed, release=True)
        self.assertEqual(failures, [])
        self.assertEqual(outcomes[0][0], "NOTE")

    def test_an_expired_exception_fails_the_release_gate(self) -> None:
        allowed = {"RUSTSEC-2026-0000": a_rust_entry(review_by="2026-11-01")}
        _, failures = rust_decide([self.row()], allowed, release=True)
        self.assertTrue(failures)

    def test_an_expired_exception_only_warns_for_a_developer(self) -> None:
        allowed = {"RUSTSEC-2026-0000": a_rust_entry(review_by="2026-11-01")}
        outcomes, failures = rust_decide([self.row()], allowed, release=False)
        self.assertEqual(failures, [])
        self.assertEqual(outcomes[0][0], "WARN")

    def test_an_exception_for_an_advisory_that_is_gone_fails_the_release_gate(self) -> None:
        allowed = {"RUSTSEC-2026-0000": a_rust_entry(review_by="2026-12-01")}
        _, failures = rust_decide([], allowed, release=True)
        self.assertTrue(failures)

    def test_an_exception_for_an_advisory_that_is_gone_only_warns_for_a_developer(self) -> None:
        allowed = {"RUSTSEC-2026-0000": a_rust_entry(review_by="2026-12-01")}
        outcomes, failures = rust_decide([], allowed, release=False)
        self.assertEqual(failures, [])
        self.assertTrue(any("did not find it" in message for _, message in outcomes))

    def test_a_patched_version_is_reported(self) -> None:
        row = self.row()
        row["patched"] = [">=2.0.0"]
        outcomes, _ = rust_decide([row], {}, release=True)
        self.assertTrue(any("patched version exists" in message for _, message in outcomes))


class TheRustFindings(unittest.TestCase):
    """Reading what cargo-audit found, in its own JSON shape."""

    def test_a_vulnerability_is_read(self) -> None:
        data = {
            "vulnerabilities": {
                "found": True,
                "count": 1,
                "list": [
                    {
                        "advisory": {"id": "RUSTSEC-2026-1111", "title": "a vulnerability"},
                        "package": {"name": "example", "version": "1.0.0"},
                        "versions": {"patched": [">=2.0.0"]},
                    }
                ],
            }
        }
        rows = rust_found(data)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["id"], "RUSTSEC-2026-1111")
        self.assertEqual(rows[0]["kind"], "vulnerability")
        self.assertEqual(rows[0]["patched"], [">=2.0.0"])

    def test_the_informational_warnings_are_flattened(self) -> None:
        data = {
            "warnings": {
                "unmaintained": [
                    {
                        "advisory": {"id": "RUSTSEC-2026-2222", "title": "unmaintained"},
                        "package": {"name": "chrono", "version": "0.4.45"},
                        "versions": {"patched": []},
                    }
                ]
            }
        }
        rows = rust_found(data)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["kind"], "unmaintained")
        self.assertEqual(rows[0]["package"], "chrono")


class TheRepositoriesOwnAllowlists(unittest.TestCase):
    """The two files the repository actually ships."""

    def test_the_npm_allowlist_satisfies_the_schema(self) -> None:
        root = HERE.parent
        entries, problems = load_allowlist(
            root / "scripts" / "npm_audit_allowlist.json",
            NPM_REQUIRED,
            "the npm advisory allowlist",
            NPM_SCOPES,
        )
        self.assertEqual(problems, [])
        self.assertIsInstance(entries, list)

    def test_the_rust_allowlist_satisfies_the_schema(self) -> None:
        root = HERE.parent
        entries, problems = load_allowlist(
            root / "scripts" / "rust_audit_allowlist.json",
            RUST_REQUIRED,
            "the Rust advisory allowlist",
            RUST_SCOPES,
        )
        self.assertEqual(problems, [])
        self.assertTrue(entries)

    def test_every_rust_exception_says_what_would_close_it(self) -> None:
        root = HERE.parent
        entries, _ = load_allowlist(
            root / "scripts" / "rust_audit_allowlist.json",
            RUST_REQUIRED,
            "the Rust advisory allowlist",
            RUST_SCOPES,
        )
        for entry in entries:
            self.assertTrue(entry["remediation"].strip(), entry["advisory"])
            self.assertTrue(entry["upstream_dependency"].strip(), entry["advisory"])

    def test_every_rust_exception_has_a_finite_review_period(self) -> None:
        root = HERE.parent
        entries, _ = load_allowlist(
            root / "scripts" / "rust_audit_allowlist.json",
            RUST_REQUIRED,
            "the Rust advisory allowlist",
            RUST_SCOPES,
        )
        for entry in entries:
            added = date.fromisoformat(entry["added"])
            review = date.fromisoformat(entry["review_by"])
            self.assertGreater(review, added, entry["advisory"])
            self.assertLessEqual((review - added).days, 90, entry["advisory"])

    def test_the_npm_allowlist_has_no_exception_without_a_review_date(self) -> None:
        root = HERE.parent
        entries, _ = load_allowlist(
            root / "scripts" / "npm_audit_allowlist.json",
            NPM_REQUIRED,
            "the npm advisory allowlist",
            NPM_SCOPES,
        )
        for entry in entries:
            self.assertTrue(entry["review_by"].strip(), entry["advisory"])
            self.assertTrue(entry["owner"].strip(), entry["advisory"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
