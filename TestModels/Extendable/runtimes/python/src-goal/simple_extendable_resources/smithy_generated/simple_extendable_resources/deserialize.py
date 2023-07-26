# Code generated by smithy-python-codegen DO NOT EDIT.

from typing import Any

from .dafny_protocol import DafnyResponse
from .errors import (
    CollectionOfErrors,
    OpaqueError,
    ServiceError,
    SimpleExtendableResourcesException,
)
from .models import (
    CreateExtendableResourceOutput,
    ExtendableResource,
    GetExtendableResourceDataOutput,
    GetExtendableResourceErrorsOutput,
    UseExtendableResourceOutput,
)
from simple.extendable.resources.internaldafny.types import (
    CreateExtendableResourceOutput_CreateExtendableResourceOutput as DafnyCreateExtendableResourceOutput,
    Error,
    Error_SimpleExtendableResourcesException,
    GetExtendableResourceDataOutput_GetExtendableResourceDataOutput as DafnyGetExtendableResourceDataOutput,
    GetExtendableResourceErrorsOutput_GetExtendableResourceErrorsOutput as DafnyGetExtendableResourceErrorsOutput,
    UseExtendableResourceOutput_UseExtendableResourceOutput as DafnyUseExtendableResourceOutput,
)

from .config import Config
from .models import (
    CreateExtendableResourceOutput,
    GetExtendableResourceDataOutput,
    GetExtendableResourceErrorsOutput,
    UseExtendableResourceOutput,
)


async def _deserialize_always_modeled_error(input: DafnyResponse, config: Config) -> GetExtendableResourceErrorsOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetExtendableResourceErrorsOutput(value=input.value.value.UnwrapOr(None),
  )

async def _deserialize_always_multiple_errors(input: DafnyResponse, config: Config) -> GetExtendableResourceErrorsOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetExtendableResourceErrorsOutput(value=input.value.value.UnwrapOr(None),
  )

async def _deserialize_always_opaque_error(input: DafnyResponse, config: Config) -> GetExtendableResourceErrorsOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetExtendableResourceErrorsOutput(value=input.value.value.UnwrapOr(None),
  )

async def _deserialize_create_extendable_resource(input: DafnyResponse, config: Config) -> CreateExtendableResourceOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return CreateExtendableResourceOutput(resource=ExtendableResource(_impl=input.value.resource),
  )

async def _deserialize_get_extendable_resource_data(input: DafnyResponse, config: Config) -> GetExtendableResourceDataOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetExtendableResourceDataOutput(blob_value=input.value.blobValue.UnwrapOr(None),
  boolean_value=input.value.booleanValue.UnwrapOr(None),
  string_value=input.value.stringValue.UnwrapOr(None),
  integer_value=input.value.integerValue.UnwrapOr(None),
  long_value=input.value.longValue.UnwrapOr(None),
  )

async def _deserialize_use_extendable_resource(input: DafnyResponse, config: Config) -> UseExtendableResourceOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return UseExtendableResourceOutput(output=GetExtendableResourceDataOutput(blob_value=input.value.output.blobValue.UnwrapOr(None),
  boolean_value=input.value.output.booleanValue.UnwrapOr(None),
  string_value=input.value.output.stringValue.UnwrapOr(None),
  integer_value=input.value.output.integerValue.UnwrapOr(None),
  long_value=input.value.output.longValue.UnwrapOr(None),
  ),
  )

async def _deserialize_use_extendable_resource_always_modeled_error(input: DafnyResponse, config: Config) -> GetExtendableResourceErrorsOutput:

  print("_deserialize_use_extendable_resource_always_modeled_error")
  print(input)
  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetExtendableResourceErrorsOutput(value=input.value.value.UnwrapOr(None),
  )

async def _deserialize_use_extendable_resource_always_multiple_errors(input: DafnyResponse, config: Config) -> GetExtendableResourceErrorsOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetExtendableResourceErrorsOutput(value=input.value.value.UnwrapOr(None),
  )

async def _deserialize_use_extendable_resource_always_opaque_error(input: DafnyResponse, config: Config) -> GetExtendableResourceErrorsOutput:

  if input.IsFailure():
    return await _deserialize_error(input.error)
  return GetExtendableResourceErrorsOutput(value=input.value.value.UnwrapOr(None),
  )

async def _deserialize_error(
    error: Error
) -> ServiceError:
  if error.is_Opaque:
    return OpaqueError(obj=error.obj)
  if error.is_CollectionOfErrors:
    return CollectionOfErrors(message=error.message, list=error.list)
  if error.is_SimpleExtendableResourcesException:
    print("is_SimpleExtendableResourcesException")
    a = SimpleExtendableResourcesException(message=error.message)
    print(a)
    return a
