fn main() {
    use pycall::MatPlotLib;
    let mut test_map = std::collections::HashMap::new();
    test_map.insert("hello".to_owned(), vec![56, 12, 65, 3, 21]);
    test_map.insert("there".to_owned(), vec![6, 2, 5, 13, 1]);
    let mut program = pycall::PythonProgram::new();
    program
        .define_variable("test_map", &test_map)
        .write_line("print(test_map)");
    program.save_as("saved.py");
}
