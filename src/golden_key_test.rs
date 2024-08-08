use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::search::path::to_path;

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
    pub fn new(input_fn: &str) -> TestFiles {
        let mut output_fn: String = input_fn.into();
        output_fn.push_str(".out");

        TestFiles {
            input_path: to_path_from_path(&Path::new(TEST_DATA_PATH).join(input_fn)),
            output_path: to_path_from_path(&Path::new(TEST_DATA_OUTPUT_PATH).join(&output_fn)),
            assert_output_path: to_path_from_path(&Path::new(TEST_DATA_PATH).join(&output_fn)),
        }
    }

    pub fn read_input(&self) -> String {
        fs::read_to_string(&self.input_path).unwrap()
    }

    pub fn read_assert_output(&self) -> String {
        fs::read_to_string(&self.assert_output_path).unwrap()
    }

    pub fn write_output<T: Debug>(&self, var: &T) {
        self.write_output_str(&TestFiles::output_string(var));
    }

    pub fn write_output_str(&self, contents: &str) {
        fs::write(&self.output_path, contents).unwrap();
    }

    pub fn assert_output<T: Debug>(&self, var: &T) -> bool {
        let output = TestFiles::output_string(var);
        self.write_output_str(&output);

        self.read_assert_output() == output
    }

    pub fn assert_output_str(&self, output: &str) -> bool {
        self.write_output_str(output);

        self.read_assert_output() == output
    }

    fn output_string<T: Debug>(var: &T) -> String {
        format!("{:?}", var)
    }
}
