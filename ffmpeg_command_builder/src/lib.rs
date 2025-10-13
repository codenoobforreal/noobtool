use std::process::Command;

#[derive(Debug, Default, Clone)]
pub struct FfmpegCommandBuilder {
    global_options: Vec<String>,
    inputs: Vec<Input>,
    output_options: Vec<String>,
    output: Option<String>,
}

#[derive(Debug, Clone)]
struct Input {
    options: Vec<String>,
    path: String,
}

impl FfmpegCommandBuilder {
    pub fn new() -> Self {
        FfmpegCommandBuilder {
            global_options: Vec::new(),
            inputs: Vec::new(),
            output_options: Vec::new(),
            output: None,
        }
    }

    fn split_long_args<S: AsRef<str>>(args: S) -> Vec<String> {
        args.as_ref()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// 添加全局选项（例如：-y）
    pub fn global_opt<S: AsRef<str>>(mut self, opt: S) -> Self {
        let parts: Vec<String> = Self::split_long_args(opt);
        self.global_options.extend(parts);
        self
    }

    /// 添加一个输入文件
    pub fn input<S: Into<String>>(mut self, path: S) -> Self {
        self.inputs.push(Input {
            options: Vec::new(),
            path: path.into(),
        });
        self
    }

    /// 为最后一个添加的输入文件添加选项（例如：-ss）
    pub fn input_opt<S: AsRef<str>>(mut self, opt: S) -> Self {
        let parts: Vec<String> = Self::split_long_args(opt);
        if let Some(input) = self.inputs.last_mut() {
            input.options.extend(parts);
        }
        self
    }

    /// 添加输出选项（例如：-c:v, -b:v）
    pub fn output_opt<S: AsRef<str>>(mut self, opt: S) -> Self {
        let parts: Vec<String> = Self::split_long_args(opt);
        self.output_options.extend(parts);
        self
    }

    /// 设置输出文件路径
    pub fn output<S: Into<String>>(mut self, path: S) -> Self {
        self.output = Some(path.into());
        self
    }

    /// 最终命令构建
    pub fn build(self) -> Command {
        let mut command = Command::new("ffmpeg");

        // 1. 全局选项
        command.args(self.global_options);

        // 2. 输入部分：对于每个输入，先加其选项，再加 -i 和路径
        for input in self.inputs {
            command.args(input.options);
            command.arg("-i");
            command.arg(input.path);
        }

        // 3. 输出选项
        command.args(self.output_options);

        // 4. 输出文件路径
        if let Some(output) = self.output {
            command.arg(output);
        }

        command
    }
}
