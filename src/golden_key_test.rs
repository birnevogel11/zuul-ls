use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::path::to_path;

const TEST_DATA_PATH: &str = "./testdata/";
const TEST_DATA_OUTPUT_PATH: &str = "./testdata/output";

fn to_path_from_path(path: &Path) -> PathBuf {
    to_path(path.to_str().unwrap())
}

#[derive(Clone, PartialEq, PartialOrd, Debug, Eq, Ord, Hash)]
pub struct TestFiles {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub assert_output_path: PathBuf,
}

impl TestFiles {
    #[allow(dead_code)]
    pub fn new(input_fn: &str) -> TestFiles {
        let mut output_fn: String = input_fn.into();
        output_fn.push_str(".out");

        TestFiles {
            input_path: to_path_from_path(&Path::new(TEST_DATA_PATH).join(input_fn)),
            output_path: to_path_from_path(&Path::new(TEST_DATA_OUTPUT_PATH).join(&output_fn)),
            assert_output_path: to_path_from_path(&Path::new(TEST_DATA_PATH).join(&output_fn)),
        }
    }

    #[allow(dead_code)]
    pub fn read_input(&self) -> String {
        fs::read_to_string(&self.input_path).unwrap()
    }

    #[allow(dead_code)]
    pub fn read_assert_output(&self) -> String {
        fs::read_to_string(&self.assert_output_path).unwrap()
    }

    #[allow(dead_code)]
    pub fn write_output<T: Debug>(&self, var: &T) {
        self.write_output_str(&TestFiles::output_string(var));
    }

    #[allow(dead_code)]
    pub fn write_output_str(&self, contents: &str) {
        fs::write(&self.output_path, contents).unwrap();
    }

    #[allow(dead_code)]
    pub fn assert_output<T: Debug>(&self, var: &T) {
        let output = TestFiles::output_string(var);
        self.write_output_str(&output);
        println!("{}", &output);

        assert_eq!(self.read_assert_output(), output)
    }

    #[allow(dead_code)]
    pub fn assert_output_str(&self, output: &str) {
        self.write_output_str(output);

        assert_eq!(self.read_assert_output(), output)
    }

    #[allow(dead_code)]
    fn output_string<T: Debug>(var: &T) -> String {
        let path = to_path(".");
        let path = path.to_str().unwrap();

        format!("{:?}", var).replace(path, ".")
    }
}
