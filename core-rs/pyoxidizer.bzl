# PyOxidizer Configuration for Sikuli-D Next Generation
# This file configures PyOxidizer to create a self-contained Python distribution

def make_exe():
    dist = default_python_distribution(python_version="3.11")

    policy = dist.make_python_packaging_policy()

    # Include all standard library modules
    policy.resources_location = "in-memory"
    policy.resources_location_fallback = "filesystem-relative:lib"

    # Allow loading of our Rust extension
    policy.allow_in_memory_shared_library_loading = True

    python_config = dist.make_python_interpreter_config()

    # Configure the interpreter
    python_config.run_command = "from sikulix import main; main()"
    python_config.module_search_paths = ["$ORIGIN/lib"]

    # Optimize for size
    python_config.optimization_level = 2
    python_config.bytecode_optimization_level = 2

    exe = dist.to_python_executable(
        name="sikulix",
        packaging_policy=policy,
        config=python_config,
    )

    # Add our extension module
    exe.add_python_extension_module(
        name="sikulix_core",
        source=FileManifest(
            files={"sikulix_core": "target/release/sikulix_core*"},
        ),
    )

    # Add the Python wrapper package
    exe.add_python_package_source(
        name="sikulix",
        root="python/sikulix",
    )

    return exe


def make_embedded_resources(exe):
    return exe.to_embedded_resources()


def make_install(exe):
    # Create installation files
    files = FileManifest()
    files.add_python_resources(exe)
    return files


# Build targets
register_target("exe", make_exe)
register_target("resources", make_embedded_resources, depends=["exe"])
register_target("install", make_install, depends=["exe"])

resolve_targets()
