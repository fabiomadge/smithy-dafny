# Code generated by smithy-python-codegen DO NOT EDIT.

from .dafny_protocol import DafnyResponse
from .models import GetBooleanOutput
from simple_types_boolean_internaldafny_types import (
    GetBooleanOutput_GetBooleanOutput as DafnyGetBooleanOutput,
)

from .config import Config
from .models import GetBooleanOutput


async def _deserialize_get_boolean(input: DafnyResponse, config: Config) -> GetBooleanOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetBooleanOutput(value=input.value.value.UnwrapOr(None),
  )
