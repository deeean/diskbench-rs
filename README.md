# diskbench
Lightweight benchmark tools for HDD, SSD

## Installation
```shell
cargo install diskbench
```

## Usage
```
diskbench --help

Flags:
        -i, --iterations <int>           : default 10
        -w, --write_buffer_size <string> : default 16.00MB
        -r, --read_buffer_size <string>  : default 16.00MB
        -t, --total_buffer_size <string> : default 1.00GB

```