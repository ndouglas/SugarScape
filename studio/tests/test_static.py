"""Checks the studio's code without running it: the Blender modules import
bpy, so no other test imports them, and a name one of them uses but no
longer defines (the title card, lost in a rewrite) only fails mid-render."""

import builtins
import pathlib
import symtable
import unittest

STUDIO = pathlib.Path(__file__).resolve().parent.parent
MODULE_NAMES = {"__file__", "__name__", "__doc__", "__spec__", "__builtins__"}


def modules():
    """Every Python file the studio runs: its own modules, the Blender
    modules and each episode's."""
    for path in sorted(STUDIO.rglob("*.py")):
        if not {"tests", "out", "__pycache__"} & set(path.relative_to(STUDIO).parts):
            yield path


def undefined_globals(source, filename="<source>"):
    """The global names `source` reads but neither defines, imports nor gets
    from builtins, each with the scope that reads it."""
    top = symtable.symtable(source, filename, "exec")
    defined = {s.get_name() for s in top.get_symbols() if s.is_assigned() or s.is_imported() or s.is_namespace()}
    known = defined | set(dir(builtins)) | MODULE_NAMES
    missing = []

    def visit(table):
        for s in table.get_symbols():
            reads_global = s.is_global() or table is top
            if s.is_referenced() and reads_global and s.get_name() not in known:
                missing.append((s.get_name(), table.get_name()))
        for child in table.get_children():
            visit(child)

    visit(top)
    return missing


class StaticTest(unittest.TestCase):
    def test_finds_a_function_that_is_called_but_no_longer_defined(self):
        source = "def build():\n    return _title_card()\n"
        self.assertEqual(undefined_globals(source), [("_title_card", "build")])

    def test_ignores_locals_parameters_closures_imports_and_builtins(self):
        source = (
            "import math\nfrom os import path\nLIMIT = 3\n"
            "class Box:\n    size = LIMIT\n"
            "def outer(a):\n    b = len(path.sep)\n"
            "    def inner():\n        return a + b + math.pi + LIMIT + Box.size\n"
            "    return [inner() for _ in range(a)]\n"
        )
        self.assertEqual(undefined_globals(source), [])

    def test_finds_a_name_read_at_module_level(self):
        self.assertEqual(undefined_globals("X = missing + 1\n"), [("missing", "top")])

    def test_every_studio_module_defines_the_globals_it_uses(self):
        for path in modules():
            with self.subTest(path=str(path.relative_to(STUDIO))):
                self.assertEqual(undefined_globals(path.read_text(), str(path)), [])


if __name__ == "__main__":
    unittest.main()
