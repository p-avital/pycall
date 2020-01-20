fn main(){
	use pycall::MatPlotLib;
	let mut program = pycall::PythonProgram::new();
	program
	.import_pyplot_as_plt()
	.plot_y(&vec![0, 1, 2, 3, 2, 1, 0])
	.plot_xyargs(&vec![0, 1, 2, 3, 4, 5, 6], &vec![0, 1, 2, 3, 2, 1, 0], "'+'")
	.show();
	program.save_as("saved.py");
}