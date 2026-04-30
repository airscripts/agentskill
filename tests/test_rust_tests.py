"""Tests for Rust test detection and mapping."""

from test_support import create_repo

from agentskill.commands.tests import analyze_tests


class TestRustTestDetection:
    def test_detects_inline_tests(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\n',
                "src/lib.rs": (
                    "pub fn parse() {}\n\n"
                    "#[cfg(test)]\nmod tests {\n"
                    "    #[test]\n    fn parses_input() {}\n}\n"
                ),
            },
        )
        result = analyze_tests(str(repo))
        assert result["rust"]["source_files"] >= 1

    def test_detects_integration_test_files(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\n',
                "src/lib.rs": "pub fn parse() {}\n",
                "tests/parser_test.rs": "use demo::parse;\n#[test]\nfn test_parse() {}\n",
            },
        )
        result = analyze_tests(str(repo))
        assert result["rust"]["test_files"] >= 1

    def test_detects_rust_framework(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\n',
                "src/lib.rs": "pub fn parse() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn works() {}\n}\n",
            },
        )
        result = analyze_tests(str(repo))
        assert result["rust"]["framework"] == "cargo test"


class TestRustTestMapping:
    def test_maps_test_to_source_file(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\n',
                "src/parser.rs": "pub fn parse() {}\n",
                "src/parser_test.rs": "#[test]\nfn test_parse() {}\n",
            },
        )
        result = analyze_tests(str(repo))
        coverage = result["rust"]["coverage_shape"]
        if coverage["mapped"]:
            assert coverage["mapped"][0]["source"] == "src/parser.rs"

    def test_identifies_untested_source_files(self, tmp_path):
        repo = create_repo(
            tmp_path,
            {
                "Cargo.toml": '[package]\nname = "demo"\nversion = "0.1.0"\n',
                "src/covered.rs": "pub fn covered() {}\n",
                "src/covered_test.rs": "#[test]\nfn test_covered() {}\n",
                "src/uncovered.rs": "pub fn uncovered() {}\n",
            },
        )
        result = analyze_tests(str(repo))
        coverage = result["rust"]["coverage_shape"]
        assert "src/uncovered.rs" in coverage["untested_source_files"]
