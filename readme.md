# `estrange`

A cli tool that moves directories from within a target to the target, making the directories in between "estranged".

## `--help`

```
Usage: estrange [OPTIONS] [SOURCES]...

Arguments:
  [SOURCES]...  The sources that we want to move into the target directory. Should be one or more sources, although this isn't validated yet

Options:
  -d, --dry-run          Applies reading operations, prints writing operations
  -t, --target <TARGET>  The directory we want to move all our sources into. Defaults to the current working directory
  -v, --verbose...       Increase logging verbosity
  -q, --quiet...         Decrease logging verbosity
  -h, --help             Print help
```

## Quick start
```sh
# Move content within `one/two/three` to the current working directory $PWD

# print directory structure
# find *
# one/two/three/four/five.file

estrange one/two/three

# find *
# four/five.file
```

## Usage

### Single argument
```sh
estrange one/two/three
```


### Multiple arguments
```sh
estrange one/two/three one/two/four
```


## Contribution

Please write a test in `test.sh` to ensure your case is covered.

## Considerations

Please consider the following before consuming this code.

- Covering my use case only unless asked otherwise.
