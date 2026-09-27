"""Unit tests for the R1/R2 repo hygiene gate in tools/hygiene_check.py.

Run directly: python tools\\hygiene_check_tests.py
All fixtures live in temp directories; nothing here touches the real repo.
"""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import hygiene_check

CHECKER_PATH = str(Path(__file__).resolve().parent / "hygiene_check.py")


def joined(*lines):
    return "\n".join(lines) + "\n"


def long_rust_function(body_count, keyword="fn big() {"):
    return joined(keyword, *(["    step();"] * body_count), "}")


def long_python_function(body_count, keyword="def big():"):
    return joined(keyword, *(["    step()"] * body_count))


def span_tuples(spans):
    return [(span.line, span.name, span.length) for span in spans]


def write_file(root, relative, text):
    target = Path(root) / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")


def run_checker(root):
    finished = subprocess.run(
        [sys.executable, CHECKER_PATH, str(root)],
        capture_output=True,
        text=True,
    )
    return finished.returncode, finished.stdout


class R2FileLengthTests(unittest.TestCase):
    def test_each_scanned_extension_flags_1001_lines_and_tolerates_1000(self):
        with tempfile.TemporaryDirectory() as root:
            for extension in ("rs", "py", "html", "js"):
                write_file(root, f"oversize.{extension}", "line\n" * 1001)
                write_file(root, f"exactly.{extension}", "line\n" * 1000)
            exit_code, stdout = run_checker(root)
        self.assertEqual(1, exit_code)
        self.assertEqual(
            [f"oversize.{extension}: R2 1001 lines > 1000"
             for extension in ("html", "js", "py", "rs")],
            stdout.splitlines(),
        )

    def test_allow_listed_report_html_is_not_reported(self):
        with tempfile.TemporaryDirectory() as root:
            write_file(root, "report.html", "line\n" * 1001)
            exit_code, stdout = run_checker(root)
        self.assertEqual(0, exit_code)
        self.assertEqual("", stdout)

    def test_pruned_directories_are_never_scanned(self):
        pruned = [
            "target", ".git", "node_modules", ".venv", "__pycache__",
            "pkg/__pycache__", ".kilo", ".code-review-graph", ".screenshot",
        ]
        with tempfile.TemporaryDirectory() as root:
            for directory in pruned:
                write_file(root, f"{directory}/oversize.rs", "line\n" * 1001)
            exit_code, stdout = run_checker(root)
        self.assertEqual(0, exit_code)
        self.assertEqual("", stdout)

    def test_ordinary_nested_directories_are_scanned(self):
        with tempfile.TemporaryDirectory() as root:
            write_file(root, "nested/deep/app.rs", "line\n" * 1001)
            exit_code, stdout = run_checker(root)
        self.assertEqual(1, exit_code)
        self.assertEqual(["nested/deep/app.rs: R2 1001 lines > 1000"],
                         stdout.splitlines())


class RustFunctionLengthTests(unittest.TestCase):
    def test_101_line_fn_flagged_and_100_line_fn_tolerated(self):
        long_spans = hygiene_check.find_rust_functions(long_rust_function(99))
        self.assertEqual([(1, "big", 101)], span_tuples(long_spans))
        short_spans = hygiene_check.find_rust_functions(long_rust_function(98))
        self.assertEqual([(1, "big", 100)], span_tuples(short_spans))

    def test_function_inside_impl_block_is_measured(self):
        text = joined(
            "impl Handler {",
            "    fn handle(&mut self) {",
            *["        step();"] * 98,
            "    }",
            "}",
        )
        self.assertEqual([(2, "handle", 100)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_function_inside_mod_tests_is_measured(self):
        text = joined(
            "mod tests {",
            "    #[test]",
            "    fn asserts_roundtrip() {",
            *["        step();"] * 99,
            "    }",
            "}",
        )
        self.assertEqual([(3, "asserts_roundtrip", 101)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_multiline_signature_starts_at_fn_keyword_line(self):
        text = joined(
            "fn compute<'a>(",
            "    input: &'a str,",
            "    factor: u32,",
            ") -> u32 {",
            "    input.len() as u32 * factor",
            "}",
        )
        self.assertEqual([(1, "compute", 6)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_nested_fn_folds_into_enclosing_fn(self):
        text = joined(
            "fn outer() {",
            "    fn inner() {",
            "        helper();",
            "    }",
            "    after();",
            "}",
        )
        self.assertEqual([(1, "outer", 6)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_pub_async_unsafe_fn_is_detected(self):
        text = long_rust_function(99, "pub async unsafe fn hazardous() {")
        self.assertEqual([(1, "hazardous", 101)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_capital_fn_in_box_dyn_is_not_a_definition(self):
        text = joined(
            "type Transformer = Box<dyn Fn(u32) -> u32>;",
            "",
            "fn apply(factory: Box<dyn Fn(u32) -> u32>) -> u32 {",
            "    factory(3)",
            "}",
        )
        self.assertEqual([(3, "apply", 3)],
                         span_tuples(hygiene_check.find_rust_functions(text)))


class RustStringAndCommentRegressionTests(unittest.TestCase):
    def test_regression_string_literal_with_triple_open_brace_does_not_extend_span(self):
        text = joined(
            "fn smuggles() {",
            '    let opener = "{{{";',
            "    run();",
            "}",
        )
        self.assertEqual([(1, "smuggles", 4)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_string_literal_with_closing_brace_does_not_end_span_early(self):
        text = joined(
            "fn closes_quietly() {",
            '    let closer = "}";',
            "    run();",
            "}",
        )
        self.assertEqual([(1, "closes_quietly", 4)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_raw_string_variants_preserve_brace_balance(self):
        text = joined(
            "fn raw_text() {",
            '    let plain = r"{";',
            '    let hashes = r#"}"#;',
            '    let double = r##"}{"##;',
            '    let multi = r#"',
            "{",
            "{",
            '"#;',
            "    done();",
            "}",
        )
        self.assertEqual([(1, "raw_text", 10)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_char_literals_do_not_affect_brace_balance(self):
        text = joined(
            "fn chars() {",
            "    let open = '{';",
            "    let close = '}';",
            "    let brace = '\\u{007B}';",
            "    let quote = '\\'';",
            "    done();",
            "}",
        )
        self.assertEqual([(1, "chars", 7)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_lifetimes_are_not_mistaken_for_char_literals(self):
        text = joined(
            "fn borrow<'a>(first: &'a str, second: &'a str) -> &'a str {",
            "    let close = '}';",
            "    if first.len() > second.len() {",
            "        return first;",
            "    }",
            "    second",
            "}",
        )
        self.assertEqual([(1, "borrow", 7)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_line_comment_braces_are_ignored(self):
        text = joined(
            "fn commented() {",
            "    // {",
            "    run(); // }",
            "    done();",
            "}",
        )
        self.assertEqual([(1, "commented", 5)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_nested_block_comments_follow_rust_nesting(self):
        text = joined(
            "fn nested() {",
            "    /* /* } */ } */",
            "    run();",
            "}",
        )
        self.assertEqual([(1, "nested", 4)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_doc_comment_braces_are_ignored(self):
        text = joined(
            "/// Returns } a { widget",
            "fn documented() {",
            "    run();",
            "}",
        )
        self.assertEqual([(2, "documented", 3)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_regression_c_string_literals_do_not_affect_balance(self):
        text = joined(
            "fn ctext() {",
            '    let s = c"{";',
            '    let raw = cr"}";',
            "    run();",
            "}",
        )
        self.assertEqual([(1, "ctext", 5)],
                         span_tuples(hygiene_check.find_rust_functions(text)))


class PythonFunctionLengthTests(unittest.TestCase):
    def test_101_line_def_flagged_and_100_line_def_tolerated(self):
        long_spans = hygiene_check.find_python_functions(long_python_function(100))
        self.assertEqual([(1, "big", 101)], span_tuples(long_spans))
        short_spans = hygiene_check.find_python_functions(long_python_function(99))
        self.assertEqual([(1, "big", 100)], span_tuples(short_spans))

    def test_decorator_lines_are_not_counted(self):
        text = joined(
            "@logged",
            "@timed",
            "def wrapped():",
            *["    step()"] * 99,
        )
        self.assertEqual([(3, "wrapped", 100)],
                         span_tuples(hygiene_check.find_python_functions(text)))

    def test_nested_def_folds_into_outer_def(self):
        text = joined(
            "def outer():",
            "    def inner():",
            "        work()",
            "    finish()",
        )
        self.assertEqual([(1, "outer", 4)],
                         span_tuples(hygiene_check.find_python_functions(text)))

    def test_def_keyword_inside_triple_quoted_string_does_not_start_measurement(self):
        text = joined(
            'SCRIPT = """',
            "def evil():",
            "    print(1)",
            '"""',
            "",
            "def real():",
            "    return 1",
        )
        self.assertEqual([(6, "real", 2)],
                         span_tuples(hygiene_check.find_python_functions(text)))

    def test_def_keyword_in_comment_does_not_start_measurement(self):
        text = joined(
            "# def archived():",
            "#     return 0",
            "def live():",
            "    pass",
        )
        self.assertEqual([(3, "live", 2)],
                         span_tuples(hygiene_check.find_python_functions(text)))

    def test_trailing_blank_lines_are_excluded_from_def_length(self):
        text = joined(
            "def ends_here():",
            "    work()",
            "",
            "",
            "sibling = 1",
        )
        self.assertEqual([(1, "ends_here", 2)],
                         span_tuples(hygiene_check.find_python_functions(text)))


class CodeReviewerRejectionTests(unittest.TestCase):
    def test_blocker1_fn_type_parameter_in_signature_does_not_split_measurement(self):
        text = joined(
            "fn transform(",
            "    value: u32,",
            "    cb: fn(u32) -> u32,",
            ") -> u32 {",
            *["    step();"] * 100,
            "}",
        )
        self.assertEqual([(1, "transform", 105)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_blocker1_capital_box_dyn_fn_type_control_not_a_definition(self):
        text = joined(
            "fn apply(factory: Box<dyn Fn(u32) -> u32>) -> u32 {",
            "    factory(3)",
            "}",
        )
        self.assertEqual([(1, "apply", 3)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_blocker1_nested_fn_folding_control(self):
        text = joined(
            "fn outer() {",
            "    fn inner() {",
            "        helper();",
            "    }",
            "    after();",
            "}",
        )
        self.assertEqual([(1, "outer", 6)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_blocker1_raw_identifier_fn_expression_cannot_start_measurement(self):
        text = joined(
            "fn target(x: u32) -> u32 { x }",
            "const CALLBACK: fn(u32) -> u32 = r#fn;",
            "fn invoke(v: u32) -> u32 { CALLBACK(v) }",
        )
        self.assertEqual([(1, "target", 1), (3, "invoke", 1)],
                         span_tuples(hygiene_check.find_rust_functions(text)))

    def test_blocker2_wrapped_def_signature_counts_full_body(self):
        wrapped = joined(
            "def big(",
            "    alpha: int,",
            "    beta: int,",
            ") -> str:",
            *["    step()"] * 101,
        )
        self.assertEqual([(1, "big", 105)],
                         span_tuples(hygiene_check.find_python_functions(wrapped)))
        clean = joined(
            "def big(",
            "    alpha: int,",
            "    beta: int,",
            ") -> str:",
            *["    step()"] * 96,
        )
        self.assertEqual([(1, "big", 100)],
                         span_tuples(hygiene_check.find_python_functions(clean)))

    def test_blocker3_column_zero_string_close_keeps_body_in_measurement(self):
        text = joined(
            "def big():",
            '    text = """',
            *["filler"] * 97,
            '"""',
            "    step()",
            "    done()",
        )
        self.assertEqual([(1, "big", 102)],
                         span_tuples(hygiene_check.find_python_functions(text)))

    def test_blocker3_real_dedent_after_string_close_still_terminates(self):
        text = joined(
            "def outer():",
            '    text = """',
            "filler",
            '"""',
            "    done()",
            "def sibling():",
            "    pass",
        )
        self.assertEqual([(1, "outer", 5), (6, "sibling", 2)],
                         span_tuples(hygiene_check.find_python_functions(text)))

    def test_blocker4_python_pseudo_breaks_count_as_one_physical_line(self):
        odd = "stuff\x0csep\u2028tail"
        with tempfile.TemporaryDirectory() as root:
            for physical in (1000, 1001):
                lines = (["TEXT = r\"\"\"", odd]
                         + ["filler"] * (physical - 4)
                         + ["\"\"\"", "TAIL = 1"])
                write_file(root, f"size{physical}.py", "\n".join(lines) + "\n")
            exit_code, stdout = run_checker(root)
        self.assertEqual(1, exit_code)
        self.assertEqual(["size1001.py: R2 1001 lines > 1000"],
                         stdout.splitlines())

    def test_blocker4_rust_pseudo_breaks_do_not_pad_fn_length(self):
        odd = "sep\u2028sep\u2028sep"

        def rust_text(step_count):
            return joined(
                "fn padded() {",
                '    let t = r"""',
                odd,
                '    """;',
                *["    step();"] * step_count,
                "}",
            )

        self.assertEqual([(1, "padded", 100)],
                         span_tuples(hygiene_check.find_rust_functions(
                             rust_text(95))))
        self.assertEqual([(1, "padded", 101)],
                         span_tuples(hygiene_check.find_rust_functions(
                             rust_text(96))))


class CliContractTests(unittest.TestCase):
    def test_clean_tree_exits_zero_without_output(self):
        with tempfile.TemporaryDirectory() as root:
            write_file(root, "clean/fine.rs", long_rust_function(98))
            write_file(root, "clean/fine.py", long_python_function(99))
            write_file(root, "report.html", "line\n" * 1001)
            exit_code, stdout = run_checker(root)
        self.assertEqual(0, exit_code)
        self.assertEqual("", stdout)

    def test_violation_tree_exits_one_and_prints_exact_lines(self):
        with tempfile.TemporaryDirectory() as root:
            write_file(root, "src/big.rs",
                       long_rust_function(99, "fn run_migration() {"))
            write_file(root, "scripts/tool.py",
                       long_python_function(100, "def execute():"))
            write_file(root, "web/page.html", "line\n" * 1001)
            exit_code, stdout = run_checker(root)
        self.assertEqual(1, exit_code)
        self.assertEqual(
            [
                "scripts/tool.py:1: R1 def `execute` is 101 lines (> 100)",
                "src/big.rs:1: R1 fn `run_migration` is 101 lines (> 100)",
                "web/page.html: R2 1001 lines > 1000",
            ],
            stdout.splitlines(),
        )

    def test_unclosed_function_reports_error_and_exits_two(self):
        with tempfile.TemporaryDirectory() as root:
            write_file(root, "broken.rs", joined("fn unclosed() {", "    run();"))
            exit_code, stdout = run_checker(root)
        self.assertEqual(2, exit_code)
        self.assertIn("ERROR", stdout)
        self.assertIn("unclosed", stdout)


if __name__ == "__main__":
    unittest.main()
