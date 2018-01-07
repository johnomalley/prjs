# Projects utility

This is a simple command-line utility to list your most recently modified projects.  It assumes that your projects are in `$HOME/projects`.  
A project is defined as a directory within with a *.git* subfolder.

## Example Output

```
$ cargo run

2018-01-06 10:01:53 rust/prjs
2017-12-31 18:10:23 folderx/projecty
2017-12-31 09:19:20 folderxyz/projectabc
```
