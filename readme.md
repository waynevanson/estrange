# `estrange`

A shell command that move child directories up their ancestors and delete the parents in between, making the directories in between "estranged".

## Dependencies

Compatible on Unix based systems that contain the following commands:

- `sh`
- `mv -t`
- `rm -r`

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

- I'm writing this to learn about POSIX scripting, so the software shouldn't be relied upon.
- Covering my use case only unless asked otherwise.
