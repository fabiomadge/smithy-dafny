# Copyright Amazon.com Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

# TODO-Python-PYTHONPATH: Qualify imports
import simple_types_blob_internaldafny_wrapped
from simple_types_blob.smithygenerated.simple_types_blob.client import SimpleTypesBlob
from simple_types_blob.smithygenerated.simple_types_blob.shim import SimpleBlobShim
from simple_types_blob.smithygenerated.simple_types_blob.config import dafny_config_to_smithy_config
import Wrappers

class default__(simple_types_blob_internaldafny_wrapped.default__):

    @staticmethod
    def WrappedSimpleBlob(config):
        wrapped_config = dafny_config_to_smithy_config(config)
        impl = SimpleTypesBlob(wrapped_config)
        wrapped_client = SimpleBlobShim(impl)
        return Wrappers.Result_Success(wrapped_client)

# (TODO-Python-PYTHONPATH: Remove)
simple_types_blob_internaldafny_wrapped.default__ = default__
