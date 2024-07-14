# `estrange`

A shell command that move child directories up their ancestors and delete the parents in between, making the directories in between "estranged".

## Dependencies

Compatible on unix based systems that contain the following commands:

- `sh`
- `mv -t`
- `rm -r`

## Usage

```sh
estrange <relative-paths-from-pwd>
```