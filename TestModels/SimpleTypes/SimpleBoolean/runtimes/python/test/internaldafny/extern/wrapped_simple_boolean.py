# Copyright Amazon.com Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

# TODO-Python-PYTHONPATH: Qualify imports
import simple_types_boolean_internaldafny_wrapped
from simple_types_boolean.smithygenerated.simple_types_boolean.client import SimpleTypesBoolean
from simple_types_boolean.smithygenerated.simple_types_boolean.shim import SimpleBooleanShim
from simple_types_boolean.smithygenerated.simple_types_boolean.config import dafny_config_to_smithy_config
import Wrappers

class default__(simple_types_boolean_internaldafny_wrapped.default__):

    @staticmethod
    def WrappedSimpleBoolean(config):
        wrapped_config = dafny_config_to_smithy_config(config)
        impl = SimpleTypesBoolean(wrapped_config)
        wrapped_client = SimpleBooleanShim(impl)
        return Wrappers.Result_Success(wrapped_client)

# (TODO-Python-PYTHONPATH: Remove)
simple_types_boolean_internaldafny_wrapped.default__ = default__
