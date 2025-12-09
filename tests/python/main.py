import ctypes
import os

ext = "so"  # Change to 'dylib' for MacOS, 'dll' for Windows
lib_path = os.path.join(
    os.path.dirname(__file__), "..", "target", "debug", f"libtests.{ext}"
)
lib = ctypes.CDLL(lib_path)
lib.__hiFfi___simple_function()
