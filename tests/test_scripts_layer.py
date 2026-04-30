from test_support import ROOT


def test_scripts_directory_contains_only_supported_wrapper_files():
    scripts_dir = ROOT / "scripts"

    names = sorted(path.name for path in scripts_dir.iterdir())

    assert names == [
        "config.py",
        "git.py",
        "graph.py",
        "measure.py",
        "scan.py",
        "symbols.py",
        "tests.py",
    ]

    for name in names:
        assert (scripts_dir / name).is_file()
