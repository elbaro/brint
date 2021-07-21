## Objects
# - make a function/method/class available or not
# -

import importlib
import inspect
import logging
import pathlib
import sys
from importlib.metadata import version
from pathlib import Path
from typing import Any, Dict, Optional, Union

import rtoml
import semver
from semver.version import Version

VersionArg = Union[semver.Version, str]

_brint_files = set()
_brint_config_cache = {}


def _find_brint_config(path: Path) -> Optional[Dict]:
    """From the caller python file, find the closest Brint.toml in parent directories

    Returns:
        Version: [description]
    """

    if path in _brint_config_cache:
        return _brint_config_cache[path]

    script_path: Path = path

    path = path.parent
    while True:
        config_path: Path = path / 'Brint.toml'
        if config_path.exists():
            # found config
            config = rtoml.load(config_path)
            _brint_config_cache[script_path] = config
            return config

        if path == path.parent:
            break

        path = path.parent

    return None


def _check_version(
    version: Version,
    current_version: Version,
) -> bool:
    return current_version.replace(prerelease=None) >= version.replace(prerelease=None)


_no_op_decorator = lambda x: x


def feature(
    name: str,
    version: str,
    old: Optional[Any] = None,
    new: Optional[Any] = None,
    author: Optional[str] = None,
    description: Optional[str] = None,
):
    version: Version = Version.parse(version)

    frame = inspect.stack()[1]
    mod = inspect.getmodule(frame[0])
    module_name = vars(sys.modules[mod.__name__])['__package__']

    root_module = importlib.import_module(module_name)

    current_version: str
    if hasattr(root_module, '__version__'):
        current_version = root_module.__version__
    elif hasattr(root_module, '__VERSION__'):
        current_version = root_module.__VERSION__
    else:
        raise RuntimeError(
            f'The module {module_name} has neither __version__ or __VERSION__'
        )

    current_version: Version = Version.parse(current_version)

    # config: Optional[Dict] = _find_brint_config(path=caller_filepath)
    # if config is None:
    #     raise RuntimeException('Brint.toml is not found')

    # version_str: str = config.get('version')
    # if version_str is None:
    #     raise RuntimeException('Brint.toml has no version field')

    if _check_version(version=version, current_version=current_version):
        if new:
            return lambda _: new
        else:
            return _no_op_decorator
    else:
        if old:
            return lambda _: old
        else:
            return _no_op_decorator
