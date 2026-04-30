"""Tests for Go test detection and mapping."""

from test_support import create_repo

from agentskill.commands.tests import analyze_tests


class TestGoTestDetection:
    def test_detects_test_go_files(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "pkg/foo.go": "package pkg\nfunc Foo() {}\n",
                "pkg/foo_test.go": 'package pkg_test\nimport "testing"\nfunc TestFoo(t *testing.T) {}\n',
            },
        )
        result = analyze_tests(str(repo))
        assert result["go"]["test_files"] == 1
        assert result["go"]["source_files"] == 1

    def test_detects_go_framework(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "main.go": "package main\n",
                "main_test.go": 'package main\nimport "testing"\nfunc TestMain(t *testing.T) {}\n',
            },
        )
        result = analyze_tests(str(repo))
        assert result["go"]["framework"] == "go test"


class TestGoTestMapping:
    def test_maps_test_to_source_file(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "pkg/foo.go": "package pkg\nfunc Foo() {}\n",
                "pkg/foo_test.go": 'package pkg_test\nimport "testing"\nfunc TestFoo(t *testing.T) {}\n',
            },
        )
        result = analyze_tests(str(repo))
        coverage = result["go"]["coverage_shape"]
        assert len(coverage["mapped"]) == 1
        assert coverage["mapped"][0]["source"] == "pkg/foo.go"
        assert coverage["mapped"][0]["test"] == "pkg/foo_test.go"

    def test_identifies_untested_source_files(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "pkg/covered.go": "package pkg\nfunc Covered() {}\n",
                "pkg/covered_test.go": 'package pkg_test\nimport "testing"\nfunc TestCovered(t *testing.T) {}\n',
                "pkg/uncovered.go": "package pkg\nfunc Uncovered() {}\n",
            },
        )
        result = analyze_tests(str(repo))
        coverage = result["go"]["coverage_shape"]
        assert "pkg/uncovered.go" in coverage["untested_source_files"]
        assert "pkg/covered.go" not in coverage["untested_source_files"]

    def test_detects_benchmark_and_example_functions(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "pkg/foo.go": "package pkg\nfunc Foo() {}\n",
                "pkg/foo_test.go": (
                    'package pkg_test\nimport "testing"\n'
                    "func TestFoo(t *testing.T) {}\n"
                    "func BenchmarkFoo(b *testing.B) {}\n"
                    "func ExampleFoo() {}\n"
                ),
            },
        )
        result = analyze_tests(str(repo))
        assert result["go"]["test_files"] == 1
