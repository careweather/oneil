from setuptools import find_packages, setup

with open("README.md", "r") as fh:
    long_description = fh.read()

setup(
    name="oneil",
    version="0.10",
    author="Patrick Walton",
    author_email="patrick@careweather.com",
    description="Design specification language for system modeling",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/careweather/oneil",
    entry_points = {
        'console_scripts': ['oneil=oneil:main'],
    },
    packages=(find_packages() + find_packages(where="./oneil") + find_packages(where="./oneil/parser")),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Programming Language :: Python :: 3",
        "Operating System :: POSIX :: Linux",
    ],
)
