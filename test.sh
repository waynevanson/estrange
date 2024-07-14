# Copy the fixtures into a temporary directory
setup() {
    rm -rf test/*
    mkdir -p test
    cp -r fixtures/* test
}

ROOT="$PWD"

estrange() {
   sh -c "$ROOT/estrange.sh "$@""
}

test() {
    local FIXTURE_PATH="$1"
    local FIXTURE_PWD="$2"
    local EXPECTED="$3"

    shift
    shift
    shift

    cd "$ROOT/$FIXTURE_PATH/$FIXTURE_PWD"

    estrange "$@"

    cd "$ROOT/$FIXTURE_PATH"

    local ACTUAL="$(find *)"

    if [ "$EXPECTED" = "$ACTUAL" ]
    then
        echo "PASS!"
    else 
        echo
        echo "FAIL: EXPECTED"
        echo "$EXPECTED"
        echo
        echo "FAIL: ACTUAL"
        echo "$ACTUAL"
    fi
    
    cd "$ROOT"
}

    
setup
read -d '\n' EXPECTED << EOF
second
second/third
second/third/sixth.file
EOF

test test/first second/third "$EXPECTED" fourth/fifth

setup
read -d '\n' EXPECTED << EOF
second
second/third
second/third/sixth.file
EOF

test test/first second/third "$EXPECTED" ./fourth/fifth
