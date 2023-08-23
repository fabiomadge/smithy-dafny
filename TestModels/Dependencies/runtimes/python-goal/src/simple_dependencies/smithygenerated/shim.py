# Code generated by smithy-python-codegen DO NOT EDIT.

from Wrappers import Option_None, Option_Some
from _dafny import Map, Seq
import module_
from simple_constraints_internaldafny_types import (
    ComplexListElement_ComplexListElement as DafnyComplexListElement,
    GetConstraintsInput_GetConstraintsInput as DafnyGetConstraintsInput,
    GetConstraintsOutput_GetConstraintsOutput as DafnyGetConstraintsOutput,
)
from simple_dependencies_internaldafny_types import (
    UseSimpleResourceInput_UseSimpleResourceInput as DafnyUseSimpleResourceInput,
)
from simple_extendable_resources_internaldafny_types import (
    GetExtendableResourceDataInput_GetExtendableResourceDataInput as DafnyGetExtendableResourceDataInput,
    GetExtendableResourceDataOutput_GetExtendableResourceDataOutput as DafnyGetExtendableResourceDataOutput,
    GetExtendableResourceErrorsInput_GetExtendableResourceErrorsInput as DafnyGetExtendableResourceErrorsInput,
    GetExtendableResourceErrorsOutput_GetExtendableResourceErrorsOutput as DafnyGetExtendableResourceErrorsOutput,
    UseExtendableResourceOutput_UseExtendableResourceOutput as DafnyUseExtendableResourceOutput,
)
from simple_resources_internaldafny_types import (
    GetResourceDataOutput_GetResourceDataOutput as DafnyGetResourceDataOutput,
    GetResourcesInput_GetResourcesInput as DafnyGetResourcesInput,
    GetResourcesOutput_GetResourcesOutput as DafnyGetResourcesOutput,
    ISimpleResource,
)

from .errors import CollectionOfErrors, OpaqueError, ServiceError
from .models import UseSimpleResourceInput
from simple_constraints.smithygenerated.models import (
    ComplexListElement,
    GetConstraintsInput,
    GetConstraintsOutput,
)
from simple_extendable_resources.smithygenerated.models import (
    GetExtendableResourceDataInput,
    GetExtendableResourceErrorsInput,
    GetExtendableResourceErrorsOutput,
    UseExtendableResourceOutput,
)
from simple_resources.smithygenerated.models import (
    GetResourceDataInput,
    GetResourceDataOutput,
    GetResourcesInput,
    GetResourcesOutput,
    SimpleResource,
)

from .errors import (
    SimpleExtendableResources
)


import Wrappers
import asyncio
import simple_dependencies_internaldafny_types
import simple_dependencies.smithygenerated.client as client_impl

from simple_extendable_resources.smithygenerated.shim import (
    smithy_error_to_dafny_error as simple_extendable_resources_smithy_error_to_dafny_error
)

def smithy_error_to_dafny_error(e: ServiceError):
    print("smithy_error_to_dafny_error")
    print(e)
    if isinstance(e, SimpleExtendableResources):
        print("isinstance of what you want")
        print(e)
        print(e.__dict__)
        return simple_dependencies_internaldafny_types.Error_SimpleExtendableResources(simple_extendable_resources_smithy_error_to_dafny_error(e.message))

    if isinstance(e, CollectionOfErrors):
        return simple_dependencies_internaldafny_types.Error_CollectionOfErrors(message=e.message, list=e.list)

    if isinstance(e, OpaqueError):
        return simple_dependencies_internaldafny_types.Error_Opaque(obj=e.obj)

class SimpleDependenciesShim(simple_dependencies_internaldafny_types.ISimpleDependenciesClient):
    def __init__(self, _impl: client_impl) :
        self._impl = _impl

    def GetSimpleResource(self, input: DafnyGetResourcesInput) -> DafnyGetResourcesOutput:
        unwrapped_request: GetResourcesInput = GetResourcesInput(value=input.value.UnwrapOr(None),
    )
        try:
            wrapped_response = asyncio.run(self._impl.get_simple_resource(unwrapped_request))
        except ServiceError as e:
            return Wrappers.Result_Failure(smithy_error_to_dafny_error(e))
        return Wrappers.Result_Success(DafnyGetResourcesOutput(output=wrapped_response.output,
    ))

    def UseSimpleResource(self, input: DafnyUseSimpleResourceInput) -> DafnyGetResourceDataOutput:
        unwrapped_request: UseSimpleResourceInput = UseSimpleResourceInput(value=SimpleResource(_impl=input.value),
    input=GetResourceDataInput(blob_value=input.input.blobValue.UnwrapOr(None),
    boolean_value=input.input.booleanValue.UnwrapOr(None),
    string_value=input.input.stringValue.UnwrapOr(None),
    integer_value=input.input.integerValue.UnwrapOr(None),
    long_value=input.input.longValue.UnwrapOr(None),
    ),
    )
        try:
            wrapped_response = asyncio.run(self._impl.use_simple_resource(unwrapped_request))
        except ServiceError as e:
            return Wrappers.Result_Failure(smithy_error_to_dafny_error(e))
        return Wrappers.Result_Success(DafnyGetResourceDataOutput(blobValue=((Option_Some(wrapped_response.blob_value)) if (wrapped_response.blob_value is not None) else (Option_None())),
    booleanValue=((Option_Some(wrapped_response.boolean_value)) if (wrapped_response.boolean_value is not None) else (Option_None())),
    stringValue=((Option_Some(wrapped_response.string_value)) if (wrapped_response.string_value is not None) else (Option_None())),
    integerValue=((Option_Some(wrapped_response.integer_value)) if (wrapped_response.integer_value is not None) else (Option_None())),
    longValue=((Option_Some(wrapped_response.long_value)) if (wrapped_response.long_value is not None) else (Option_None())),
    ))

    def UseLocalConstraintsService(self, input: DafnyGetConstraintsInput) -> DafnyGetConstraintsOutput:
        print('shimming')
        unwrapped_request: GetConstraintsInput = GetConstraintsInput(my_string=input.MyString.UnwrapOr(None),
    non_empty_string=input.NonEmptyString.UnwrapOr(None),
    string_less_than_or_equal_to_ten=input.StringLessThanOrEqualToTen.UnwrapOr(None),
    my_blob=input.MyBlob.UnwrapOr(None),
    non_empty_blob=input.NonEmptyBlob.UnwrapOr(None),
    blob_less_than_or_equal_to_ten=input.BlobLessThanOrEqualToTen.UnwrapOr(None),
    my_list=[list_element for list_element in input.MyList.UnwrapOr(None)],
    non_empty_list=[list_element for list_element in input.NonEmptyList.UnwrapOr(None)],
    list_less_than_or_equal_to_ten=[list_element for list_element in input.ListLessThanOrEqualToTen.UnwrapOr(None)],
    my_map={key: value for (key, value) in input.MyMap.UnwrapOr(None).items },
    non_empty_map={key: value for (key, value) in input.NonEmptyMap.UnwrapOr(None).items },
    map_less_than_or_equal_to_ten={key: value for (key, value) in input.MapLessThanOrEqualToTen.UnwrapOr(None).items },
    alphabetic=input.Alphabetic.UnwrapOr(None),
    one_to_ten=input.OneToTen.UnwrapOr(None),
    greater_than_one=input.GreaterThanOne.UnwrapOr(None),
    less_than_ten=input.LessThanTen.UnwrapOr(None),
    my_unique_list=[list_element for list_element in input.MyUniqueList.UnwrapOr(None)],
    my_complex_unique_list=[ComplexListElement(value=list_element.value.UnwrapOr(None),
    blob=list_element.blob.UnwrapOr(None),
    ) for list_element in input.MyComplexUniqueList.UnwrapOr(None)],
    my_utf8_bytes=input.MyUtf8Bytes.UnwrapOr(None),
    my_list_of_utf8_bytes=[list_element for list_element in input.MyListOfUtf8Bytes.UnwrapOr(None)],
    )
        print("unwrapped")
        try:
            wrapped_response = asyncio.run(self._impl.use_local_constraints_service(unwrapped_request))
            print("wrapped response here")
        except ServiceError as e:
            return Wrappers.Result_Failure(smithy_error_to_dafny_error(e))
        return Wrappers.Result_Success(DafnyGetConstraintsOutput(MyString=((Option_Some(wrapped_response.my_string)) if (wrapped_response.my_string is not None) else (Option_None())),
    NonEmptyString=((Option_Some(wrapped_response.non_empty_string)) if (wrapped_response.non_empty_string is not None) else (Option_None())),
    StringLessThanOrEqualToTen=((Option_Some(wrapped_response.string_less_than_or_equal_to_ten)) if (wrapped_response.string_less_than_or_equal_to_ten is not None) else (Option_None())),
    MyBlob=((Option_Some(wrapped_response.my_blob)) if (wrapped_response.my_blob is not None) else (Option_None())),
    NonEmptyBlob=((Option_Some(wrapped_response.non_empty_blob)) if (wrapped_response.non_empty_blob is not None) else (Option_None())),
    BlobLessThanOrEqualToTen=((Option_Some(wrapped_response.blob_less_than_or_equal_to_ten)) if (wrapped_response.blob_less_than_or_equal_to_ten is not None) else (Option_None())),
    MyList=((Option_Some(Seq([list_element for list_element in wrapped_response.my_list]))) if (wrapped_response.my_list is not None) else (Option_None())),
    NonEmptyList=((Option_Some(Seq([list_element for list_element in wrapped_response.non_empty_list]))) if (wrapped_response.non_empty_list is not None) else (Option_None())),
    ListLessThanOrEqualToTen=((Option_Some(Seq([list_element for list_element in wrapped_response.list_less_than_or_equal_to_ten]))) if (wrapped_response.list_less_than_or_equal_to_ten is not None) else (Option_None())),
    MyMap=((Option_Some(Map({key: value for (key, value) in wrapped_response.my_map.items() }))) if (wrapped_response.my_map is not None) else (Option_None())),
    NonEmptyMap=((Option_Some(Map({key: value for (key, value) in wrapped_response.non_empty_map.items() }))) if (wrapped_response.non_empty_map is not None) else (Option_None())),
    MapLessThanOrEqualToTen=((Option_Some(Map({key: value for (key, value) in wrapped_response.map_less_than_or_equal_to_ten.items() }))) if (wrapped_response.map_less_than_or_equal_to_ten is not None) else (Option_None())),
    Alphabetic=((Option_Some(wrapped_response.alphabetic)) if (wrapped_response.alphabetic is not None) else (Option_None())),
    OneToTen=((Option_Some(wrapped_response.one_to_ten)) if (wrapped_response.one_to_ten is not None) else (Option_None())),
    GreaterThanOne=((Option_Some(wrapped_response.greater_than_one)) if (wrapped_response.greater_than_one is not None) else (Option_None())),
    LessThanTen=((Option_Some(wrapped_response.less_than_ten)) if (wrapped_response.less_than_ten is not None) else (Option_None())),
    MyUniqueList=((Option_Some(Seq([list_element for list_element in wrapped_response.my_unique_list]))) if (wrapped_response.my_unique_list is not None) else (Option_None())),
    MyComplexUniqueList=((Option_Some(Seq([DafnyComplexListElement(value=((Option_Some(list_element.value)) if (list_element.value is not None) else (Option_None())),
    blob=((Option_Some(list_element.blob)) if (list_element.blob is not None) else (Option_None())),
    ) for list_element in wrapped_response.my_complex_unique_list]))) if (wrapped_response.my_complex_unique_list is not None) else (Option_None())),
    MyUtf8Bytes=((Option_Some(wrapped_response.my_utf8_bytes)) if (wrapped_response.my_utf8_bytes is not None) else (Option_None())),
    MyListOfUtf8Bytes=((Option_Some(Seq([list_element for list_element in wrapped_response.my_list_of_utf8_bytes]))) if (wrapped_response.my_list_of_utf8_bytes is not None) else (Option_None())),
    ))

    def UseLocalExtendableResource(self, input: DafnyGetExtendableResourceDataInput) -> DafnyUseExtendableResourceOutput:
        unwrapped_request: GetExtendableResourceDataInput = GetExtendableResourceDataInput(blob_value=input.blobValue.UnwrapOr(None),
    boolean_value=input.booleanValue.UnwrapOr(None),
    string_value=input.stringValue.UnwrapOr(None),
    integer_value=input.integerValue.UnwrapOr(None),
    long_value=input.longValue.UnwrapOr(None),
    )
        try:
            wrapped_response = asyncio.run(self._impl.use_local_extendable_resource(unwrapped_request))
        except ServiceError as e:
            return Wrappers.Result_Failure(smithy_error_to_dafny_error(e))
        return Wrappers.Result_Success(DafnyUseExtendableResourceOutput(output=DafnyGetExtendableResourceDataOutput(blobValue=((Option_Some(wrapped_response.output.blob_value)) if (wrapped_response.output.blob_value is not None) else (Option_None())),
    booleanValue=((Option_Some(wrapped_response.output.boolean_value)) if (wrapped_response.output.boolean_value is not None) else (Option_None())),
    stringValue=((Option_Some(wrapped_response.output.string_value)) if (wrapped_response.output.string_value is not None) else (Option_None())),
    integerValue=((Option_Some(wrapped_response.output.integer_value)) if (wrapped_response.output.integer_value is not None) else (Option_None())),
    longValue=((Option_Some(wrapped_response.output.long_value)) if (wrapped_response.output.long_value is not None) else (Option_None())),
    ),
    ))

    def LocalExtendableResourceAlwaysModeledError(self, input: DafnyGetExtendableResourceErrorsInput) -> DafnyGetExtendableResourceErrorsOutput:
        unwrapped_request: GetExtendableResourceErrorsInput = GetExtendableResourceErrorsInput(value=input.value.UnwrapOr(None),
    )
        try:
            wrapped_response = asyncio.run(self._impl.local_extendable_resource_always_modeled_error(unwrapped_request))
            print("wrapped_response")
            print(wrapped_response)
        except ServiceError as e:
            print("got a serviceerror")
            print(e)
            print(type(e))
            return Wrappers.Result_Failure(smithy_error_to_dafny_error(e))
        return Wrappers.Result_Success(DafnyGetExtendableResourceErrorsOutput(value=((Option_Some(wrapped_response.value)) if (wrapped_response.value is not None) else (Option_None())),
    ))

    def LocalExtendableResourceAlwaysMultipleErrors(self, input: DafnyGetExtendableResourceErrorsInput) -> DafnyGetExtendableResourceErrorsOutput:
        unwrapped_request: GetExtendableResourceErrorsInput = GetExtendableResourceErrorsInput(value=input.value.UnwrapOr(None),
    )
        try:
            wrapped_response = asyncio.run(self._impl.local_extendable_resource_always_multiple_errors(unwrapped_request))
        except ServiceError as e:
            return Wrappers.Result_Failure(smithy_error_to_dafny_error(e))
        return Wrappers.Result_Success(DafnyGetExtendableResourceErrorsOutput(value=((Option_Some(wrapped_response.value)) if (wrapped_response.value is not None) else (Option_None())),
    ))

    def LocalExtendableResourceAlwaysNativeError(self, input: DafnyGetExtendableResourceErrorsInput) -> DafnyGetExtendableResourceErrorsOutput:
        unwrapped_request: GetExtendableResourceErrorsInput = GetExtendableResourceErrorsInput(value=input.value.UnwrapOr(None),
    )
        try:
            wrapped_response = asyncio.run(self._impl.local_extendable_resource_always_native_error(unwrapped_request))
        except ServiceError as e:
            return Wrappers.Result_Failure(smithy_error_to_dafny_error(e))
        return Wrappers.Result_Success(DafnyGetExtendableResourceErrorsOutput(value=((Option_Some(wrapped_response.value)) if (wrapped_response.value is not None) else (Option_None())),
    ))
