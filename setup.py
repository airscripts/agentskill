from setuptools import setup, find_packages

setup(
    name="agentskill",
    version="1.0.0",
    description="Generate AGENTS.md from actual coding style",
    author="Clawdeeo",
    packages=find_packages(),
    entry_points={
        "console_scripts": [
            "agentskill=agentskill.cli:main",
        ],
    },
    python_requires=">=3.8",
)
