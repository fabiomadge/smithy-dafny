# Code generated by smithy-python-codegen DO NOT EDIT.
#
# from .simple_types_boolean_internaldafny_types import (
#     GetBooleanInput_GetBooleanInput as DafnyGetBooleanInput,
# )


from . import Wrappers
from typing import Union

class DafnyRequest:
    # from .simple_types_boolean_internaldafny_types import (
    #     GetBooleanInput_GetBooleanInput as DafnyGetBooleanInput,
    # )

    operation_name: str
    dafny_operation_input: any

    def __init__(self, operation_name, dafny_operation_input):
        self.operation_name = operation_name
        self.dafny_operation_input = dafny_operation_input

class DafnyResponse(Wrappers.Result):
    def __init__(self):
        super.__init__(self)
