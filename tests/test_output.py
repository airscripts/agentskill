import json

from lib.output import run_and_output, write_output


def test_write_output_prints_and_writes_file(tmp_path, capsys):
    write_output({"ok": True}, pretty=False)
    assert json.loads(capsys.readouterr().out) == {"ok": True}

    out_file = tmp_path / "report.json"
    write_output({"ok": True}, pretty=True, out=str(out_file))

    assert json.loads(out_file.read_text()) == {"ok": True}


def test_run_and_output_handles_success_and_exceptions(tmp_path, capsys):
    def ok(repo: str, *, value: int):
        return {"repo": repo, "value": value}

    exit_code = run_and_output(
        ok,
        repo="sample",
        pretty=False,
        script_name="demo",
        extra_kwargs={"value": 3},
    )

    assert exit_code == 0
    assert json.loads(capsys.readouterr().out) == {"repo": "sample", "value": 3}

    out_file = tmp_path / "error.json"

    def boom(repo: str):
        raise RuntimeError("kaboom")

    exit_code = run_and_output(
        boom,
        repo="sample",
        pretty=True,
        out=str(out_file),
        script_name="demo",
    )

    assert exit_code == 1

    assert json.loads(out_file.read_text()) == {"error": "kaboom", "script": "demo"}
