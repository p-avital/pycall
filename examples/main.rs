fn main() {
    let handles = (
        std::thread::spawn(|| pycall::plot!(&vec![0, 1, 2, 3, 2, 1, 0])),
        std::thread::spawn(|| {
            pycall::plot!(&vec![-3, -2, -1, 3, 4, 5, 6], &vec![0, 1, 2, 3, 2, 1, 0])
        }),
        std::thread::spawn(|| {
            pycall::plot!(&vec![0, 1, 2, 3, 4, 5, 6], &vec![0, 1, 2, 3, 2, 1, 0], "+")
        }),
    );
    use pycall::MatPlotLib;
    let mut program = pycall::PythonProgram::new();
    program
        .import_pyplot_as_plt()
        .plot_y(&vec![0, 1, 2, 3, 2, 1, 0])
        .plot_xyargs(&vec![0, 1, 2, 3, 4, 5, 6], &vec![0, 1, 2, 3, 2, 1, 0], "+")
        .show();
    program.background_run();
    handles.0.join();
    handles.1.join();
    handles.2.join();
}
