use regex::Regex;

// pub use file_cache; // TODO make a lib for TestResult, import it both in file-cache and test-utils

#[macro_export]
macro_rules! expect {
    ($is_true:expr $(,)?) => {
        match (&$is_true) {
            is_true_val => {
                if !(is_true_val) {
                    return Err(anyhow::anyhow!("expected true: {}", stringify!($is_true),))?;
                }
            }
        }
    };
}
#[macro_export]
macro_rules! expect_eq {
    ($left:expr, $right:expr $(, $msg:expr)?) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let mut err_str = format!(
                        "not equal:\n\tleft: {} = {:?}\n\tright: {} = {:?}\n\t",
                        stringify!($left), left_val,
                        stringify!($right), right_val
                    );
                    $(
                        err_str.push_str(&format!("{}", $msg));
                    )*
                    Err(anyhow::Error::msg(err_str))?;
                }
            }
        }
    };
}

pub type TestResult = Result<(), TestError>;
pub struct TestError(pub anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for TestError {
    fn from(e: E) -> Self {
        TestError(e.into())
    }
}
impl std::fmt::Debug for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\x1b[0;91m{}\x1b[0m", self.0)?;

        // print only the items of the backtrace that originate in our files, not rust-std or dependencies
        let re = Regex::new(r#"(?:Backtrace\s*\[\s*)?\{\s*fn:\s*"([^"]+)",\s*file:\s*"([^"]+)",\s*line:\s*(\d+)\s*\}\s*(?:\]\s*)?"#).unwrap();
        re.captures_iter(&format!("{:#?}", self.0.backtrace()))
            .filter_map(|cap| cap.get(2).map(|path| (cap, path.as_str().to_owned()))) // only where has field file
            .map(|(cap3, path)| TestBacktraceFrame {
                fn_name: cap3.get(1).map(|m| m.as_str().to_owned()),
                file: path,
                line: cap3.get(3).and_then(|m| m.as_str().parse::<usize>().ok()),
            })
            .filter(|btf| btf.file.starts_with("./")) // only our files
            .for_each(|frame| {
                writeln!(f, "{frame:?}").ok();
            });

        Ok(())
    }
}

struct TestBacktraceFrame {
    pub fn_name: Option<String>,
    pub file: String,
    pub line: Option<usize>,
}

impl std::fmt::Debug for TestBacktraceFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}/{}:{}] {}",
            std::env::current_dir().unwrap().to_string_lossy(),
            self.file.trim_start_matches("./"),
            self.line.map(|n| n.to_string()).unwrap_or_default(),
            self.fn_name.as_ref().unwrap_or(&String::new())
        )
    }
}
