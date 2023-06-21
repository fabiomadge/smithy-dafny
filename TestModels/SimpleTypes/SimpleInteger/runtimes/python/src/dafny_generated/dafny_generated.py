# Dafny program the_program compiled into Python
import sys
from typing import Callable, Any, TypeVar, NamedTuple
from math import floor
from itertools import count

from extern import Extern

import module_
import _dafny

try:
    dafnyArgs = [_dafny.Seq(a) for a in sys.argv]
    module_.default__.Test____Main____(dafnyArgs)
except _dafny.HaltException as e:
    _dafny.print("[Program halted] " + e.message + "\n")
    sys.exit(1)
