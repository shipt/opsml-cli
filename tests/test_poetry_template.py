import os
from importlib.machinery import SourceFileLoader
from importlib.metadata import version
from importlib.util import module_from_spec, spec_from_loader


def test_version() -> None:
    assert version("poetry-template") == "2023.5.22"


def test_main() -> None:
    loader = SourceFileLoader("__main__", os.path.join("poetry_template", "__main__.py"))

    spec = spec_from_loader(loader.name, loader)
    assert spec is not None

    mod = module_from_spec(spec)
    assert mod is not None

    loader.exec_module(mod)
