# -*- mode: python ; coding: utf-8 -*-
"""PyInstaller spec for aurotype-engine sidecar binary.

Build with:  cd engine && uv run pyinstaller aurotype-engine.spec --noconfirm
Output:      engine/dist/aurotype-engine/aurotype-engine
"""

from PyInstaller.utils.hooks import collect_data_files, collect_submodules
import sys
use_upx = sys.platform != 'darwin'

# sounddevice ships portaudio shared libs that must be bundled
sounddevice_datas = collect_data_files("sounddevice")
sounddevice_data_datas = collect_data_files("_sounddevice_data")
dashscope_datas = collect_data_files("dashscope")

a = Analysis(
    ["aurotype_engine/__main__.py"],
    pathex=[],
    binaries=[],
    datas=[
        *sounddevice_datas,
        *sounddevice_data_datas,
        *dashscope_datas,
    ],
    hiddenimports=[
        # uvicorn internals (auto-detected poorly)
        "uvicorn.logging",
        "uvicorn.loops",
        "uvicorn.loops.auto",
        "uvicorn.protocols",
        "uvicorn.protocols.http",
        "uvicorn.protocols.http.auto",
        "uvicorn.protocols.http.h11_impl",
        "uvicorn.protocols.http.httptools_impl",
        "uvicorn.protocols.websockets",
        "uvicorn.protocols.websockets.auto",
        "uvicorn.protocols.websockets.wsproto_impl",
        "uvicorn.lifespan",
        "uvicorn.lifespan.on",
        "uvicorn.lifespan.off",
        # ASGI / web
        "engineio",
        "multipart",
        "multipart.multipart",
        # audio
        "sounddevice",
        "_sounddevice_data",
        # numpy
        "numpy",
        "numpy.core",
        "numpy.core._methods",
        "numpy.lib",
        "numpy.lib.format",
        # dashscope
        "dashscope",
        # httpx / httpcore
        "httpx",
        "httpcore",
        "h11",
        # pydantic
        "pydantic",
        "pydantic_settings",
        # openai
        "openai",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
)

pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="aurotype-engine",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=use_upx,
    upx_exclude=[],
    console=False,
)
