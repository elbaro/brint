# Brint

Brint is a feature gating library without feature flags.  
Each feature is associated with a version, and automatically enabled depending on the current module version.

### Example
See [example/test.py](https://github.com/elbaro/brint/blob/main/example/test.py).

```sh
pip install brint
python -m example.test
```

### brint-py

`@brint.feature(..)` decorates functions and classes.  
You can override an old entity with new one, or the new with the old.


```py
import brint

# functions

def get_dark_default_theme() -> str:  # experimental function
    return 'dark'

@brint.feature(name='dark default theme', version='1.0.0', new=get_dark_default_theme)
def get_default_theme() -> str:
    return 'light'

# classes

class LegacyProcessor(): ...

@brint.feature(name='multi-threaded processor', version='1.0.0', old=LegacyProcessor)
class Processor(): ...
```

### brint-cli

These features can be listed in the command line.

```sh
❯ brint

Last
    [0.9.2]
    ..

Current
    [1.0.0]
        new "dark default theme"
            - def default_theme in ./main.py:20
            - def web_default_theme in ./user.py:30

        new "multi-threaded processor"
            - class Processor in ./main.py:50

        obsolete "temporary feature1"
        obsolete "temporary feature2"


Upcoming
    [1.0.1]
    ..

    [1.0.2]
    ..
```

### Pre-release and Build metadata
Pre-release is intentionally ignored when checking the version.
Brint will complain about any pre-release in the condition.

```py
@brint.feature('1')  # this featute is included in 1.0.0-alpha.1, 1.0.0-beta, 1.0.0
@brint.feature('1.0.0-beta.1')  # error
@brint.feature('1.0.0+20210101')  # same as '1.0.0'. build metadata is ignored
```

### `Brint.toml` (TODO)

Feature flags can be defined in `Brint.toml`.  
This is useful for (1) sharing feature flags in a cross-language codebase, and (2) managing feature flags in one place.


```toml
[features]
feature-name-1 = "1.0.0"
feature-name-2 = { version = "2", author = "@dev", description = "desc" }
```

```py
@brint.feature('feature-name-1', new=new_fn)
def old_fn(): ...
```
