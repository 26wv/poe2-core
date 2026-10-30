# Generates the YAML fixtures for locate/yaml.rs with PyYAML, the way Lutris writes them
# (docs/research/linux-paths.md Q9; ADR 0005). Run once from the repo root:
#   python3 -m venv .venv
#   .venv/bin/pip install pyyaml==6.0.3
#   .venv/bin/python crates/poe2-core/src/locate/fixtures/gen_yaml.py
import os

import yaml

assert yaml.__version__ == "6.0.3", yaml.__version__

EXE = "/home/<user>/Games/path-of-exile-2/drive_c/Program Files (x86)/Grinding Gear Games/Path of Exile 2/PathOfExile.exe"

CASES = {
    "lutris_plain": {
        "game": {"exe": EXE, "prefix": "/home/<user>/Games/path-of-exile-2", "arch": "win64"},
        "name": "Path of Exile 2",
        "game_slug": "path-of-exile-2",
        "runner": "wine",
        "script": {"game": {"exe": "$GAMEDIR/drive_c/PathOfExile.exe", "prefix": "$GAMEDIR"}},
    },
    "lutris_folded": {
        "game": {
            "prefix": "/home/<user>/Games/a folder name long enough that PyYAML folds this plain scalar onto a second line/path-of-exile-2"
        }
    },
    "lutris_unicode": {
        "game": {
            "prefix": "/home/<user>/Jeux/Éditions spéciales/préfixe — Path of Exile 2 with enough extra words to wrap the line"
        }
    },
    "lutris_quoted": {"game": {"prefix": "/home/<user>/Games/PoE #2's prefix"}},
    "lutris_type_like": {"game": {"prefix": "yes"}},
    "lutris_script_only": {"name": "Path of Exile 2", "script": {"game": {"prefix": "$GAMEDIR"}}},
    "bottles_placeholder": {"Path": "/mnt/games/bottles/PoE2"},
    "bottles_data": {"custom_bottles_path": "/mnt/games/bottles"},
}

here = os.path.dirname(os.path.abspath(__file__))
for name, data in CASES.items():
    with open(os.path.join(here, name + ".yml"), "w", encoding="utf-8", newline="\n") as f:
        f.write(yaml.safe_dump(data, default_flow_style=False))
