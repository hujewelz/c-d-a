# cda
代码重复分析器。

## 安装

1. 安装 Rust
  ```shell
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
   
## 使用
 
 安装完 Rust 环境后，将代码下载到本地。

 进入项目根目录下，执行如下命令运行程序。

通过 `--help` 来查看 `cda` 程序帮助。

```
cargo run -- --help
```

- -r: 指定文件夹根目录。
- -s: 指定要比较的代码目录。
- -d: 指定比较的目标代码目录。


```
cargo run -- -r /path/to/RootDir -s /path/to/RootDir/newProject -d /path/to/RootDir/oldProject
```
上面的示例指定了代码分析根目录为 _/path/to/RootDir_ ，新代码的路径为 _/path/to/RootDir/newProject_, 老代码路径为 _/path/to/RootDir/oldProject_。

为了程序能输出正确的结果，我们可以先将要比较的两份源代码代码放入 `-r` 制定的统一目录下。例如:

```
| compare a-b
|----a
|----b
```