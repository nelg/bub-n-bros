#! /usr/bin/env python3

from setuptools import setup, Extension

setup ( name="xshm",
        version="0.3",
        description="X window system Shared Memory extension",
        author="Armin & Odie",
        author_email="arigo@tunes.org",
        ext_modules=[Extension(name = 'xshm',
                               sources = ['xshm.c'],
                               libraries = ['X11', 'Xext', 'Xrender'])]
        )
