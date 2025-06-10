# git-digger

Helper library to deal with multiple repositories from multiple git hosting services.


## Who uses it?


* [Public mdbooks](https://mdbooks.code-maven.com/) - [source](https://github.com/szabgab/mdbooks.code-maven.com)
* [rust-digger](https://rust-digger.code-maven.com/) - [source](https://github.com/szabgab/rust-digger/)

One day it might become useful for others as well.


## Release process

* Update the version number in Cargo.toml
* `cargo build`
* `git add .`
* `git commit -m "update version number"`
* `cargo publish`
