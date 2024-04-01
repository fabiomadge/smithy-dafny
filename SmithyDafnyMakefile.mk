# Copyright Amazon.com Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

# These are make targets that can be shared between all projects
# that use a common layout.
# They will only function if executed inside a project directory.
# See https://github.com/smithy-lang/smithy-dafny/tree/main-1.x/TestModels
# for examples.

# There are several variables that are used here.
# The expectation is to define these variables
# inside each project.

# Variables:
# MAX_RESOURCE_COUNT -- The Dafny report generator max resource count.
# 	This is is per project because the verification variability can differ.
# VERIFY_TIMEOUT -- The Dafny verification timeout in seconds.
# 	This is only a guard against builds taking way too long to fail.
#   The resource count limit above is much more important for fighting brittle verification.
# PROJECT_DEPENDENCIES -- List of dependencies for the project.
# 	It should be the list of top level directory names
# PROJECT_SERVICES -- List of names of each local service in the project
# SERVICE_NAMESPACE_<service> -- for each service in PROJECT_SERVICES,
#   the list of dependencies for that smithy namespace. It should be a list
#   of Model directories
# SERVICE_DEPS_<service> -- for each service in PROJECT_SERVICES,
#   the list of dependencies for that smithy namespace. It should be a list
#   of Model directories
# AWS_SDK_CMD -- the `--aws-sdk` command to generate AWS SDK style interfaces.
# STD_LIBRARY -- path from this file to the StandardLibrary Dafny project.
# SMITHY_DEPS -- path from this file to smithy dependencies, such as custom traits.
# GRADLEW -- the gradlew to use when building Java runtimes.
#   defaults to $(SMITHY_DAFNY_ROOT)/codegen/gradlew

MAX_RESOURCE_COUNT := 10000000
VERIFY_TIMEOUT := 100

# This evaluates to the path of the current working directory.
# i.e. The specific library under consideration.
LIBRARY_ROOT := $(PWD)
# Smithy Dafny code gen needs to know
# where the smithy model is.
# This is generally in the same directory as the library.
# However in the case of a wrapped library,
# such as the test vectors
# the implementation MAY be in a different library
# than the model.
# By having two related variables
# test vector projects can point to
# the specific model they need
# but still build everything in their local library directory.
SMITHY_MODEL_ROOT := $(LIBRARY_ROOT)/Model

CODEGEN_CLI_ROOT := $(SMITHY_DAFNY_ROOT)/codegen/smithy-dafny-codegen-cli
GRADLEW := $(SMITHY_DAFNY_ROOT)/codegen/gradlew

# On macOS, sed requires an extra parameter of ""
OS := $(shell uname)
ifeq ($(OS), Darwin)
  SED_PARAMETER := ""
else
  SED_PARAMETER :=
endif

########################## Dafny targets

# TODO: This target will not work for projects that use `replaceable` 
#       module syntax with multiple language targets.
# It will fail with error:
# Error: modules 'A' and 'B' both have CompileName 'same.extern.name'
# We need to come up with some way to verify files per-language.
# Rewrite this as part of Java implementation of LanguageSpecificLogic TestModel.

# Proof of correctness for the math below
#  function Z3_PROCESSES(cpus:nat): nat
#  { if cpus >= 3 then 2 else 1 }

#  function DAFNY_PROCESSES(cpus: nat): nat
#   requires 0 < cpus // 0 cpus would do no work!
#  { (cpus - 1 )/Z3_PROCESSES(cpus) }

#  lemma Correct(cpus:nat)
#    ensures DAFNY_PROCESSES(cpus) * Z3_PROCESSES(cpus) <= cpus
#  {}

# Verify the entire project
verify:Z3_PROCESSES=$(shell echo $$(( $(CORES) >= 3 ? 2 : 1 )))
verify:DAFNY_PROCESSES=$(shell echo $$(( ($(CORES) - 1 ) / ($(CORES) >= 3 ? 2 : 1))))
verify:
	find . -name '*.dfy' | xargs -n 1 -P $(DAFNY_PROCESSES) -I % dafny \
		-vcsCores:$(Z3_PROCESSES) \
		-compile:0 \
		-definiteAssignment:3 \
		-unicodeChar:0 \
		-functionSyntax:3 \
		-verificationLogger:csv \
		-timeLimit:$(VERIFY_TIMEOUT) \
		-trace \
		%

# Verify single file FILE with text logger.
# This is useful for debugging resource count usage within a file.
# Use PROC to further scope the verification
verify_single:
	dafny \
		-vcsCores:$(CORES) \
		-compile:0 \
		-definiteAssignment:3 \
		-unicodeChar:0 \
		-functionSyntax:3 \
		-verificationLogger:text \
		-timeLimit:$(VERIFY_TIMEOUT) \
		-trace \
		$(if ${PROC},-proc:*$(PROC)*,) \
		$(FILE)

#Verify only a specific namespace at env var $(SERVICE)
verify_service:
	@: $(if ${SERVICE},,$(error You must pass the SERVICE to generate for));
	dafny \
		-vcsCores:$(CORES) \
		-compile:0 \
		-definiteAssignment:3 \
		-unicodeChar:0 \
		-functionSyntax:3 \
		-verificationLogger:csv \
		-timeLimit:$(VERIFY_TIMEOUT) \
		-trace \
		`find ./dafny/$(SERVICE) -name '*.dfy'` \

format_dafny:
	dafny format \
		--function-syntax 3 \
		--unicode-char false \
		`find . -name '*.dfy'`

format_dafny-check:
	dafny format \
		--check \
		--function-syntax 3 \
		--unicode-char false \
		`find . -name '*.dfy'`

dafny-reportgenerator:
	dafny-reportgenerator \
		summarize-csv-results \
		--max-resource-count $(MAX_RESOURCE_COUNT) \
		TestResults/*.csv

clean-dafny-report:
	rm TestResults/*.csv

# Dafny helper targets

# Transpile the entire project's impl
# For each index file listed in the project Makefile's PROJECT_INDEX variable,
#   append a `-library:TestModels/$(PROJECT_INDEX) to the transpiliation target
_transpile_implementation_all: TRANSPILE_DEPENDENCIES=$(patsubst %, -library:$(PROJECT_ROOT)/%, $(PROJECT_INDEX))
_transpile_implementation_all: transpile_implementation

# The `$(OUT)` and $(TARGET) variables are problematic.
# Ideally they are different for every target call.
# However the way make evaluates variables
# having a target specific variable is hard.
# This all comes up because a project
# will need to also transpile its dependencies.
# This is worked around for now,
# by the fact that the `TARGET`
# for all these transpile calls will be the same.
# For `OUT` this is solved by making the paths relative.
# So that the runtime is express once
# and can be the same for all such runtimes.
# Since such targets are all shared,
# this is tractable.

# If the project under transpilation uses `replaceable` modules,
#   it MUST define a SRC_INDEX variable per language.
# SRC_INDEX points to the folder containing the `Index.dfy` file for a particular language
#   that `include`s all of that language's `replaces` modules.
# This variable's value might look like (ex.) `src/replaces/net` or `src/replaces/java`.
# If this variable is not provided, assume the project does not have `replaceable` modules,
#   and look for `Index.dfy` in the `src/` directory.
transpile_implementation: SRC_INDEX_TRANSPILE=$(if $(SRC_INDEX),$(SRC_INDEX),src)
transpile_implementation:
    ifeq ($(TARGET), py)
        COMPILE_SUFFIX_OPTION := -compileSuffix:0
    else
        COMPILE_SUFFIX_OPTION := -compileSuffix:1
    endif
# At this time it is *significatly* faster
# to give Dafny a single file
# that includes everything
# than it is to pass each file to the CLI.
# ~2m vs ~10s for our large projects.
# Also the expectation is that verification happens in the `verify` target
# `find` looks for `Index.dfy` files in either V1 or V2-styled project directories (single vs. multiple model files).
transpile_implementation:
	find ./dafny/**/$(SRC_INDEX_TRANSPILE)/ ./$(SRC_INDEX_TRANSPILE)/ -name 'Index.dfy' | sed -e 's/^/include "/' -e 's/$$/"/' | dafny \
		-stdin \
		-noVerify \
		-vcsCores:$(CORES) \
		-compileTarget:$(TARGET) \
		-spillTargetCode:3 \
		-compile:0 \
		-optimizeErasableDatatypeWrapper:0 \
		$(COMPILE_SUFFIX_OPTION) \
		-unicodeChar:0 \
		-functionSyntax:3 \
		-useRuntimeLib \
		-out $(OUT) \
		$(if $(strip $(STD_LIBRARY)) , -library:$(PROJECT_ROOT)/$(STD_LIBRARY)/src/Index.dfy, ) \
		$(TRANSPILE_DEPENDENCIES)

# If the project under transpilation uses `replaceable` modules,
#   it MUST define a SRC_INDEX variable per language.
# The purpose and usage of this is described in the `transpile_implementation` comments.
_transpile_test_all: SRC_INDEX_TRANSPILE=$(if $(SRC_INDEX),$(SRC_INDEX),src)
# If the project under transpilation uses `replaceable` modules in its tests
#   it MUST define a TEST_INDEX variable per language.
# TEST_INDEX points to the folder containing all test files for a particular language.
# These files should use Dafny `include`s to include the generic test files as well.
# This variable's value might look like (ex.) `test/replaces/net` or `test/replaces/java`.
# If this variable is not provided, assume the project does not have `replaceable` modules,
#   and look for test files in the `test/` directory.
_transpile_test_all: TEST_INDEX_TRANSPILE=$(if $(TEST_INDEX),$(TEST_INDEX),test)
# If the Makefile defines DIR_STRUCTURE_V2 (i.e. multiple models/subprojects/services in project), then:
#   For each of this project's services defined in PROJECT_SERVICES:
#     append `-library:/path/to/Index.dfy` to the transpile target
# Else: (i.e. single model/service in project), then:
#   append `-library:/path/to/Index.dfy` to the transpile target
_transpile_test_all: TRANSPILE_DEPENDENCIES=$(if ${DIR_STRUCTURE_V2}, $(patsubst %, -library:dafny/%/$(SRC_INDEX_TRANSPILE)/Index.dfy, $(PROJECT_SERVICES)), -library:$(SRC_INDEX_TRANSPILE)/Index.dfy)
# Transpile the entire project's tests
_transpile_test_all: transpile_test

# `find` looks for tests in either V1 or V2-styled project directories (single vs. multiple model files).
transpile_test:
    ifeq ($(TARGET), py)
        COMPILE_SUFFIX_OPTION := -compileSuffix:0
    else
        COMPILE_SUFFIX_OPTION := -compileSuffix:1
    endif
transpile_test:
	find ./dafny/**/$(TEST_INDEX_TRANSPILE) ./$(TEST_INDEX_TRANSPILE) -name "*.dfy" -name '*.dfy' | sed -e 's/^/include "/' -e 's/$$/"/' | dafny \
		-stdin \
		-noVerify \
		-vcsCores:$(CORES) \
		-compileTarget:$(TARGET) \
		-spillTargetCode:3 \
		-runAllTests:1 \
		-compile:0 \
		-optimizeErasableDatatypeWrapper:0 \
		$(COMPILE_SUFFIX_OPTION) \
		-unicodeChar:0 \
		-functionSyntax:3 \
		-useRuntimeLib \
		-out $(OUT) \
		$(if $(strip $(STD_LIBRARY)) , -library:$(PROJECT_ROOT)/$(STD_LIBRARY)/src/Index.dfy, ) \
		$(TRANSPILE_DEPENDENCIES) \

# If we are not the StandardLibrary, transpile the StandardLibrary.
# Transpile all other dependencies
transpile_dependencies:
	$(if $(strip $(STD_LIBRARY)), $(MAKE) -C $(PROJECT_ROOT)/$(STD_LIBRARY) transpile_implementation_$(LANG), )
	$(patsubst %, $(MAKE) -C $(PROJECT_ROOT)/% transpile_implementation_$(LANG);, $(PROJECT_DEPENDENCIES))

########################## Code-Gen targets

# The OUTPUT variables are created this way
# so that it is possible to run _parts_ of polymorph.
# Otherwise it is difficult to run/test only a Dafny change.
# Since they are defined per target
# a single target can decide what parts it wants to build.

# Pass in CODEGEN_CLI_ROOT in command line, e.g.
#   make polymorph_code_gen CODEGEN_CLI_ROOT=/path/to/smithy-dafny/codegen/smithy-dafny-codegen-cli
# StandardLibrary is filtered out from dependent-model patsubst list;
#   Its model is contained in $(LIBRARY_ROOT)/model, not $(LIBRARY_ROOT)/../StandardLibrary/Model.
_polymorph:
	cd $(CODEGEN_CLI_ROOT); \
	./../gradlew run --args="\
	--dafny-version $(DAFNY_VERSION) \
	--library-root $(LIBRARY_ROOT) \
	--patch-files-dir $(if $(DIR_STRUCTURE_V2),$(LIBRARY_ROOT)/codegen-patches/$(SERVICE),$(LIBRARY_ROOT)/codegen-patches) \
	--properties-file $(LIBRARY_ROOT)/project.properties \
	$(INPUT_DAFNY) \
	$(OUTPUT_DAFNY) \
	$(OUTPUT_JAVA) \
	$(OUTPUT_DOTNET) \
	$(OUTPUT_JAVA) \
	$(OUTPUT_PYTHON) \
	$(MODULE_NAME) \
	--model $(if $(DIR_STRUCTURE_V2), $(LIBRARY_ROOT)/dafny/$(SERVICE)/Model, $(SMITHY_MODEL_ROOT)) \
	--dependent-model $(PROJECT_ROOT)/$(SMITHY_DEPS) \
	$(patsubst %, --dependent-model $(PROJECT_ROOT)/%/Model, $($(service_deps_var))) \
	$(DEPENDENCY_MODULE_NAMES) \
	--namespace $($(namespace_var)) \
	$(OUTPUT_LOCAL_SERVICE_$(SERVICE)) \
	$(AWS_SDK_CMD) \
	$(POLYMORPH_OPTIONS) \
	";

_polymorph_wrapped:
	@: $(if ${CODEGEN_CLI_ROOT},,$(error You must pass the path CODEGEN_CLI_ROOT: CODEGEN_CLI_ROOT=/path/to/smithy-dafny/codegen/smithy-dafny-codegen-cli));
	cd $(CODEGEN_CLI_ROOT); \
	./../gradlew run --args="\
	--dafny-version $(DAFNY_VERSION) \
	--library-root $(LIBRARY_ROOT) \
	--properties-file $(LIBRARY_ROOT)/project.properties \
	$(OUTPUT_DAFNY_WRAPPED) \
	$(OUTPUT_DOTNET_WRAPPED) \
	$(OUTPUT_JAVA_WRAPPED) \
	--model $(if $(DIR_STRUCTURE_V2),$(LIBRARY_ROOT)/dafny/$(SERVICE)/Model,$(LIBRARY_ROOT)/Model) \
	--dependent-model $(PROJECT_ROOT)/$(SMITHY_DEPS) \
	$(patsubst %, --dependent-model $(PROJECT_ROOT)/%/Model, $($(service_deps_var))) \
	--namespace $($(namespace_var)) \
	--local-service-test \
	$(AWS_SDK_CMD) \
	$(POLYMORPH_OPTIONS)";

_polymorph_dependencies:
	@$(foreach dependency, \
		$(PROJECT_DEPENDENCIES), \
		$(MAKE) -C $(PROJECT_ROOT)/$(dependency) polymorph_$(POLYMORPH_LANGUAGE_TARGET); \
	)

# Generates all target runtime code for all namespaces in this project.
.PHONY: polymorph_code_gen
polymorph_code_gen: POLYMORPH_LANGUAGE_TARGET=code_gen
polymorph_code_gen: _polymorph_dependencies
polymorph_code_gen:
	set -e; for service in $(PROJECT_SERVICES) ; do \
		export service_deps_var=SERVICE_DEPS_$${service} ; \
		export namespace_var=SERVICE_NAMESPACE_$${service} ; \
		export SERVICE=$${service} ; \
		$(MAKE) _polymorph_code_gen ; \
	done

_polymorph_code_gen: OUTPUT_DAFNY=\
    --output-dafny $(if $(DIR_STRUCTURE_V2), $(LIBRARY_ROOT)/dafny/$(SERVICE)/Model, $(LIBRARY_ROOT)/Model)
_polymorph_code_gen: INPUT_DAFNY=\
		--include-dafny $(PROJECT_ROOT)/$(STD_LIBRARY)/src/Index.dfy
_polymorph_code_gen: OUTPUT_DOTNET=\
    $(if $(DIR_STRUCTURE_V2), --output-dotnet $(LIBRARY_ROOT)/runtimes/net/Generated/$(SERVICE)/, --output-dotnet $(LIBRARY_ROOT)/runtimes/net/Generated/)
_polymorph_code_gen: OUTPUT_JAVA=--output-java $(LIBRARY_ROOT)/runtimes/java/src/main/smithy-generated
_polymorph_code_gen: _polymorph

check_polymorph_diff:
	git diff --exit-code $(LIBRARY_ROOT) || (echo "ERROR: polymorph-generated code does not match the committed code - see above for diff. Either commit the changes or regenerate with 'POLYMORPH_OPTIONS=--update-patch-files'." && exit 1)

# Generates dafny code for all namespaces in this project
.PHONY: polymorph_dafny
polymorph_dafny: POLYMORPH_LANGUAGE_TARGET=dafny
polymorph_dafny: _polymorph_dependencies
polymorph_dafny:
	set -e; for service in $(PROJECT_SERVICES) ; do \
		export service_deps_var=SERVICE_DEPS_$${service} ; \
		export namespace_var=SERVICE_NAMESPACE_$${service} ; \
		export SERVICE=$${service} ; \
		$(MAKE) _polymorph_dafny ; \
	done

_polymorph_dafny: OUTPUT_DAFNY=\
		--output-dafny $(if $(DIR_STRUCTURE_V2), $(LIBRARY_ROOT)/dafny/$(SERVICE)/Model, $(LIBRARY_ROOT)/Model)
_polymorph_dafny: INPUT_DAFNY=\
		--include-dafny $(PROJECT_ROOT)/$(STD_LIBRARY)/src/Index.dfy
_polymorph_dafny: _polymorph

# Generates dotnet code for all namespaces in this project
.PHONY: polymorph_dotnet
polymorph_dotnet: POLYMORPH_LANGUAGE_TARGET=dotnet
polymorph_dotnet: _polymorph_dependencies
polymorph_dotnet:
	set -e; for service in $(PROJECT_SERVICES) ; do \
		export service_deps_var=SERVICE_DEPS_$${service} ; \
		export namespace_var=SERVICE_NAMESPACE_$${service} ; \
		export SERVICE=$${service} ; \
		$(MAKE) _polymorph_dotnet ; \
	done

_polymorph_dotnet: OUTPUT_DOTNET=\
    $(if $(DIR_STRUCTURE_V2), --output-dotnet $(LIBRARY_ROOT)/runtimes/net/Generated/$(SERVICE)/, --output-dotnet $(LIBRARY_ROOT)/runtimes/net/Generated/)
_polymorph_dotnet: _polymorph

# Generates java code for all namespaces in this project
.PHONY: polymorph_java
polymorph_java: POLYMORPH_LANGUAGE_TARGET=java
polymorph_java: _polymorph_dependencies
polymorph_java:
	set -e; for service in $(PROJECT_SERVICES) ; do \
		export service_deps_var=SERVICE_DEPS_$${service} ; \
		export namespace_var=SERVICE_NAMESPACE_$${service} ; \
		export SERVICE=$${service} ; \
		$(MAKE) _polymorph_java ; \
	done

_polymorph_java: OUTPUT_JAVA=--output-java $(LIBRARY_ROOT)/runtimes/java/src/main/smithy-generated
_polymorph_java: _polymorph

# Generates python code for all namespaces in this project
.PHONY: polymorph_python
polymorph_python: POLYMORPH_LANGUAGE_TARGET=python
polymorph_python: _polymorph_dependencies
polymorph_python:
	set -e; for service in $(PROJECT_SERVICES) ; do \
		export service_deps_var=SERVICE_DEPS_$${service} ; \
		export namespace_var=SERVICE_NAMESPACE_$${service} ; \
		export SERVICE=$${service} ; \
		$(MAKE) _polymorph_python ; \
	done

_polymorph_python: OUTPUT_PYTHON=--output-python $(LIBRARY_ROOT)/runtimes/python/smithygenerated
_polymorph_python: _polymorph

# Dependency for formatting generating Java code
setup_prettier:
	npm i --no-save prettier@3 prettier-plugin-java@2.5

########################## .NET targets

transpile_net: | transpile_implementation_net transpile_test_net transpile_dependencies_net

transpile_implementation_net: TARGET=cs
transpile_implementation_net: OUT=runtimes/net/ImplementationFromDafny
transpile_implementation_net: SRC_INDEX=$(NET_SRC_INDEX)
transpile_implementation_net: _transpile_implementation_all

transpile_test_net: SRC_INDEX=$(NET_SRC_INDEX)
transpile_test_net: TEST_INDEX=$(NET_TEST_INDEX)
transpile_test_net: TARGET=cs
transpile_test_net: OUT=runtimes/net/tests/TestsFromDafny
transpile_test_net: _transpile_test_all

transpile_dependencies_net: LANG=net
transpile_dependencies_net: transpile_dependencies

test_net: FRAMEWORK=net6.0
test_net:
	dotnet run \
		--project runtimes/net/tests/ \
		--framework $(FRAMEWORK)

test_net_mac_intel: FRAMEWORK=net6.0
test_net_mac_intel:
	DYLD_LIBRARY_PATH="/usr/local/opt/openssl@1.1/lib" dotnet run \
		--project runtimes/net/tests/ \
		--framework $(FRAMEWORK)

test_net_mac_brew: FRAMEWORK=net6.0
test_net_mac_brew:
	DYLD_LIBRARY_PATH="$(shell brew --prefix)/opt/openssl@1.1/lib/" dotnet run \
		--project runtimes/net/tests/ \
		--framework $(FRAMEWORK)

setup_net:
	dotnet restore runtimes/net/

format_net:
	dotnet format runtimes/net/*.csproj

format_net-check:
	dotnet format runtimes/net/*.csproj --verify-no-changes

########################## Java targets

build_java: transpile_java mvn_local_deploy_dependencies
	$(GRADLEW) -p runtimes/java build

transpile_java: | transpile_implementation_java transpile_test_java transpile_dependencies_java

transpile_implementation_java: TARGET=java
transpile_implementation_java: OUT=runtimes/java/ImplementationFromDafny
transpile_implementation_java: _transpile_implementation_all _mv_implementation_java

transpile_test_java: TARGET=java
transpile_test_java: OUT=runtimes/java/TestsFromDafny
transpile_test_java: _transpile_test_all _mv_test_java

# Currently Dafny compiles to Java by changing the directory name.
# Java puts things under a `java` directory.
# To avoid `java/implementation-java` the code is generated and then moved.
_mv_implementation_java:
	rm -rf runtimes/java/src/main/dafny-generated
	mv runtimes/java/ImplementationFromDafny-java runtimes/java/src/main/dafny-generated
_mv_test_java:
	rm -rf runtimes/java/src/test/dafny-generated
	mkdir -p runtimes/java/src/test
	mv runtimes/java/TestsFromDafny-java runtimes/java/src/test/dafny-generated

transpile_dependencies_java: LANG=java
transpile_dependencies_java: transpile_dependencies

# If we are not StandardLibrary, locally deploy the StandardLibrary.
# Locally deploy all other dependencies 
mvn_local_deploy_dependencies:
	$(if $(strip $(STD_LIBRARY)), $(MAKE) -C $(PROJECT_ROOT)/$(STD_LIBRARY) mvn_local_deploy, )
	$(patsubst %, $(MAKE) -C $(PROJECT_ROOT)/% mvn_local_deploy;, $(PROJECT_DEPENDENCIES))

# The Java MUST all exist already through the transpile step.
mvn_local_deploy:
	$(GRADLEW) -p runtimes/java publishMavenLocalPublicationToMavenLocal

# The Java MUST all exsist if we want to publish to CodeArtifact
mvn_ca_deploy:
	$(GRADLEW) -p runtimes/java publishMavenPublicationToPublishToCodeArtifactCIRepository

mvn_staging_deploy:
	$(GRADLEW) -p runtimes/java publishMavenPublicationToPublishToCodeArtifactStagingRepository

test_java:
	$(GRADLEW) -p runtimes/java runTests

_clean:
	rm -f $(LIBRARY_ROOT)/Model/*Types.dfy $(LIBRARY_ROOT)/Model/*TypesWrapped.dfy
	rm -f $(LIBRARY_ROOT)/runtimes/net/ImplementationFromDafny.cs
	rm -f $(LIBRARY_ROOT)/runtimes/net/tests/TestFromDafny.cs
	rm -rf $(LIBRARY_ROOT)/TestResults
	rm -rf $(LIBRARY_ROOT)/runtimes/net/Generated $(LIBRARY_ROOT)/runtimes/net/bin $(LIBRARY_ROOT)/runtimes/net/obj
	rm -rf $(LIBRARY_ROOT)/runtimes/net/tests/bin $(LIBRARY_ROOT)/runtimes/net/tests/obj

clean: _clean

########################## Python targets

transpile_python: _python_underscore_dependency_extern_names
transpile_python: _python_underscore_extern_names
transpile_python: transpile_implementation_python
transpile_python: transpile_test_python
transpile_python: transpile_dependencies_python
transpile_python: _python_revert_underscore_extern_names
transpile_python: _python_revert_underscore_dependency_extern_names
transpile_python: _mv_internaldafny_python
transpile_python: _remove_src_module_python
transpile_python: _rename_test_main_python

transpile_implementation_python: TARGET=py
transpile_implementation_python: OUT=runtimes/python/dafny_src
transpile_implementation_python: COMPILE_SUFFIX_OPTION=
transpile_implementation_python: SRC_INDEX=$(PYTHON_SRC_INDEX)
transpile_implementation_python: _transpile_implementation_all
transpile_implementation_python: transpile_dependencies_python
transpile_implementation_python: transpile_src_python
transpile_implementation_python: transpile_test_python
transpile_implementation_python: _mv_internaldafny_python
transpile_implementation_python: _remove_src_module_python

transpile_src_python: TARGET=py
transpile_src_python: OUT=runtimes/python/dafny_src
transpile_src_python: COMPILE_SUFFIX_OPTION=
transpile_src_python: SRC_INDEX=$(PYTHON_SRC_INDEX)
transpile_src_python: _transpile_implementation_all

transpile_test_python: TARGET=py
transpile_test_python: OUT=runtimes/python/__main__
transpile_test_python: COMPILE_SUFFIX_OPTION=
transpile_test_python: SRC_INDEX=$(PYTHON_SRC_INDEX)
transpile_test_python: TEST_INDEX=$(PYTHON_TEST_INDEX)
transpile_test_python: _transpile_test_all

# Hacky workaround until Dafny supports per-language extern names.
# Replaces `.`s with `_`s in strings like `{:extern ".*"}`.
# This is flawed logic and should be removed, but is a reasonable band-aid for now.
# TODO-Python BLOCKING: Once Dafny supports per-language extern names, remove and replace with Pythonic extern names.
# This is tracked in https://github.com/dafny-lang/dafny/issues/4322.
# This may require new Smithy-Dafny logic to generate Pythonic extern names.
_python_underscore_extern_names:
	find $(if ${DIR_STRUCTURE_V2},dafny/**/src,src)  -regex ".*\.dfy" -type f -exec sed -i $(SED_PARAMETER) '/.*{:extern \".*\".*/s/\./_/g' {} \;
	find $(if ${DIR_STRUCTURE_V2},dafny/**/Model,Model) -regex ".*\.dfy" -type f -exec sed -i $(SED_PARAMETER) '/.*{:extern \".*\.*"/s/\./_/g' {} \;
	find $(if ${DIR_STRUCTURE_V2},dafny/**/test,test) -regex ".*\.dfy" -type f -exec sed -i $(SED_PARAMETER) '/.*{:extern \".*\".*/s/\./_/g' {} \;

_python_underscore_dependency_extern_names:
	$(MAKE) -C $(PROJECT_ROOT)/$(STD_LIBRARY) _python_underscore_extern_names
	@$(foreach dependency, \
		$(PROJECT_DEPENDENCIES), \
		$(MAKE) -C $(PROJECT_ROOT)/$(dependency) _python_underscore_extern_names; \
	)

_python_revert_underscore_extern_names:
	find $(if ${DIR_STRUCTURE_V2},dafny/**/src,src) -regex ".*\.dfy" -type f -exec sed -i $(SED_PARAMETER) '/.*{:extern \".*\".*/s/_/\./g' {} \;
	find $(if ${DIR_STRUCTURE_V2},dafny/**/Model,Model)  -regex ".*\.dfy" -type f -exec sed -i $(SED_PARAMETER) '/.*{:extern \".*\".*/s/_/\./g' {} \; 2>/dev/null
	find $(if ${DIR_STRUCTURE_V2},dafny/**/test,test) -regex ".*\.dfy" -type f -exec sed -i $(SED_PARAMETER) '/.*{:extern \".*\".*/s/_/\./g' {} \;

_python_revert_underscore_dependency_extern_names:
	$(MAKE) -C $(PROJECT_ROOT)/$(STD_LIBRARY) _python_revert_underscore_extern_names
	@$(foreach dependency, \
		$(PROJECT_DEPENDENCIES), \
		$(MAKE) -C $(PROJECT_ROOT)/$(dependency) _python_revert_underscore_extern_names; \
	)

# Move Dafny-generated code into its expected location in the Python module
_mv_internaldafny_python:
	# Remove any previously generated Dafny code in src/, then copy in newly-generated code
	rm -rf runtimes/python/src/$(PYTHON_MODULE_NAME)/internaldafny/generated/
	mkdir -p runtimes/python/src/$(PYTHON_MODULE_NAME)/internaldafny/generated/
	mv runtimes/python/dafny_src-py/*.py runtimes/python/src/$(PYTHON_MODULE_NAME)/internaldafny/generated
	rm -rf runtimes/python/dafny_src-py
	# Remove any previously generated Dafny code in test/, then copy in newly-generated code
	rm -rf runtimes/python/test/internaldafny/generated
	mkdir -p runtimes/python/test/internaldafny/generated
	mv runtimes/python/__main__-py/*.py runtimes/python/test/internaldafny/generated
	rm -rf runtimes/python/__main__-py

# Versions of Dafny as of ~9/28 seem to ALWAYS write output to __main__.py,
#   regardless of the OUT parameter...?
# We should figure out what happened and get a workaround
# For now, always write OUT to __main__, then manually rename the primary file...
# TODO-Python BLOCKING: Resolve this before releasing libraries
# Note the name internaldafny_test_executor is specifically chosen
# so as to not be picked up by pytest,
# which finds test_*.py or *_test.py files.
# This is neither, and will not be picked up by pytest.
# This file SHOULD not be run from a context that has not imported the wrapping shim,
#   otherwise the tests will fail.
# We write an extern which WILL be picked up by pytest.
# This extern will import the wrapping shim, then import this `internaldafny_test_executor` to run the tests.
_rename_test_main_python:
	mv runtimes/python/test/internaldafny/generated/__main__.py runtimes/python/test/internaldafny/generated/internaldafny_test_executor.py

_remove_src_module_python:
	# Remove the src/ `module_.py` file.
	# There is a race condition between the src/ and test/ installation of this file.
	# The file that is installed least recently is overwritten by the file installed most recently.
	# The test/ file contains code to execute tests. The src/ file is largely empty.
	# If the src/ file is installed most recently, tests will fail to run.
	# By removing the src/ file, we ensure the test/ file is always the installed file.
	rm runtimes/python/src/$(PYTHON_MODULE_NAME)/internaldafny/generated/module_.py

transpile_dependencies_python: LANG=python
transpile_dependencies_python: transpile_dependencies

test_python:
	rm -rf runtimes/python/.tox
	tox -c runtimes/python --verbose

########################## local testing targets

# These targets are added as a convenience for local development.
# If you experience weird behavior using these targets,
# fall back to the regular `build` or `test` targets.
# These targets MUST only be used for local testing,
# and MUST NOT be used in CI.

# Targets to transpile single local service for convenience.
# Specify the local service to build by passing a SERVICE env var.
# Note that this does not generate identical files as `transpile_implementation_java`

local_transpile_impl_java_single: TARGET=java
local_transpile_impl_java_single: OUT=runtimes/java/ImplementationFromDafny
local_transpile_impl_java_single: local_transpile_impl_single
	cp -R runtimes/java/ImplementationFromDafny-java/* runtimes/java/src/main/dafny-generated
	rm -rf runtimes/java/ImplementationFromDafny-java/

local_transpile_impl_net_single: TARGET=cs
local_transpile_impl_net_single: OUT=runtimes/net/ImplementationFromDafny
local_transpile_impl_net_single: local_transpile_impl_single

local_transpile_impl_single: deps_var=SERVICE_DEPS_$(SERVICE)
local_transpile_impl_single: TRANSPILE_TARGETS=./dafny/$(SERVICE)/src/$(FILE)
local_transpile_impl_single: TRANSPILE_DEPENDENCIES= \
		$(patsubst %, -library:$(PROJECT_ROOT)/%/src/Index.dfy, $($(deps_var))) \
		$(patsubst %, -library:$(PROJECT_ROOT)/%, $(PROJECT_INDEX)) \
		-library:$(PROJECT_ROOT)/$(STD_LIBRARY)/src/Index.dfy
local_transpile_impl_single: transpile_implementation

# Targets to transpile single local service for convenience.
# Specify the local service to build by passing a SERVICE env var.
# Note that this does not generate identical files as `transpile_test_java`,
# and will clobber tests generated by other services.

local_transpile_test_java_single: TARGET=java
local_transpile_test_java_single: OUT=runtimes/java/TestsFromDafny
local_transpile_test_java_single: local_transpile_test_single
	cp -R runtimes/java/TestsFromDafny-java/* runtimes/java/src/test/dafny-generated
	rm -rf runtimes/java/TestsFromDafny-java

local_transpile_test_net_single: TARGET=cs
local_transpile_test_net_single: OUT=runtimes/net/tests/TestsFromDafny
local_transpile_test_net_single: local_transpile_test_single

local_transpile_test_single: TRANSPILE_TARGETS=./dafny/$(SERVICE)/test/$(FILE)
local_transpile_test_single: TRANSPILE_DEPENDENCIES= \
		$(patsubst %, -library:dafny/%/src/Index.dfy, $(PROJECT_SERVICES)) \
		$(patsubst %, -library:$(PROJECT_ROOT)/%, $(PROJECT_INDEX)) \
		-library:$(PROJECT_ROOT)/$(STD_LIBRARY)/src/Index.dfy
local_transpile_test_single: transpile_test
