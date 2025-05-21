# Some helper scripts using the 'Just' script runner
# See https://github.com/casey/just for more details

default: run

run *args="../../examples/oneil_cylinder.on":
    cd src/oneil && python3 -c "import __init__; __init__.main()" {{args}}