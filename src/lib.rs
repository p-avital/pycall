use std::fmt::{Display, Error, Formatter};
use std::io::Write;

pub trait AsPythonLitteral {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

macro_rules! as_py_lit_impl {
    ($t: ty, $fmt_str: expr) => {
        impl AsPythonLitteral for $t {
            fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
                write!(f, $fmt_str, &self)
            }
        }
    };
}

as_py_lit_impl!(str, "\"{}\"");
as_py_lit_impl!(u8, "{}");
as_py_lit_impl!(u16, "{}");
as_py_lit_impl!(u32, "{}");
as_py_lit_impl!(u64, "{}");
as_py_lit_impl!(u128, "{}");
as_py_lit_impl!(usize, "{}");
as_py_lit_impl!(i8, "{}");
as_py_lit_impl!(i16, "{}");
as_py_lit_impl!(i32, "{}");
as_py_lit_impl!(i64, "{}");
as_py_lit_impl!(i128, "{}");
as_py_lit_impl!(isize, "{}");
as_py_lit_impl!(f32, "{:.6e}");
as_py_lit_impl!(f64, "{:.6e}");

impl<T: AsPythonLitteral> AsPythonLitteral for Vec<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[")?;
        for x in self.iter() {
            write!(f, "{},", PythonLitteral(x))?;
        }
        write!(f, "]")
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Indents(pub isize);

impl std::fmt::Display for Indents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for _ in 0..self.0 {
            write!(f, "\t")?;
        }
        Ok(())
    }
}

pub struct PythonProgram {
    file: tempfile::NamedTempFile,
    indents: Indents,
}

struct PythonLitteral<'l, T: AsPythonLitteral + ?Sized>(pub &'l T);
impl<'l, T: AsPythonLitteral + ?Sized> Display for PythonLitteral<'l, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

impl PythonProgram {
    pub fn new() -> PythonProgram {
        PythonProgram {
            file: tempfile::NamedTempFile::new().unwrap(),
            indents: Indents(0),
        }
    }

    pub fn run(&self) -> Result<std::process::Output, std::io::Error> {
        std::process::Command::new("python3")
            .arg(self.file.path())
            .output()
    }

    pub fn flush(&mut self) -> &mut Self {
        self.file.flush().unwrap();
        self
    }

    fn indent(&mut self, n: isize) -> &mut Self {
        if n >= 0 {
            self.indents.0 += n as isize
        } else {
            self.indents.0 -= n as isize
        }
        self
    }

    pub fn end_block(&mut self) -> &mut Self {
        self.indent(-1)
    }

    pub fn define_variable<T: AsPythonLitteral + ?Sized>(
        &mut self,
        name: &str,
        value: &T,
    ) -> &mut Self {
        writeln!(
            &mut self.file,
            "{}{} = {}",
            self.indents,
            name,
            PythonLitteral(value)
        )
        .unwrap();
        self
    }

    pub fn write_line(&mut self, line: &str) -> &mut Self {
        writeln!(&mut self.file, "{}{}", self.indents, line).unwrap();
        self
    }

    pub fn r#if(&mut self, condition: &str) -> &mut Self {
        writeln!(&mut self.file, "{}if {}:", self.indents, condition).unwrap();
        self.indent(1)
    }
    pub fn elif(&mut self, condition: &str) -> &mut Self {
        self.indent(-1);
        writeln!(&mut self.file, "{}elif {}:", self.indents, condition).unwrap();
        self.indent(1)
    }
    pub fn r#else(&mut self) -> &mut Self {
        self.indent(-1).write_line("else:").indent(1)
    }

    pub fn r#for(&mut self, range: &str) -> &mut Self {
        writeln!(&mut self.file, "{}for {}:", self.indents, range).unwrap();
        self.indent(1)
    }

    pub fn r#while(&mut self, condition: &str) -> &mut Self {
        writeln!(&mut self.file, "{}while {}:", self.indents, condition).unwrap();
        self.indent(1)
    }
}

impl Write for PythonProgram {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.file.flush()
    }
}

impl Display for PythonProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        use std::io::BufRead;
        let read_file = std::fs::File::open(self.file.path()).unwrap();
        let reader = std::io::BufReader::new(read_file);
        for line in reader.lines() {
            writeln!(f, "{}", line.unwrap())?
        }
        Ok(())
    }
}
pub mod plots {
    use std::io::Write;
    use crate::{AsPythonLitteral, PythonProgram, PythonLitteral};

    pub fn plot_xyargs<X: AsPythonLitteral, Y: AsPythonLitteral>(
        x: &X,
        y: &Y,
        args: &str,
    ) -> Result<std::process::Output, std::io::Error> {
        let mut program = PythonProgram::new();
        program.write_line("import matplotlib.pyplot as plt");
        writeln!(
            &program.file,
            "plt.plot({}, {}, {})",
            PythonLitteral(x),
            PythonLitteral(y),
            PythonLitteral(args)
        );
        program.write_line("plt.show()").run()
    }

    pub fn plot_xy<X: AsPythonLitteral, Y: AsPythonLitteral>(
        x: &X,
        y: &Y,
    ) -> Result<std::process::Output, std::io::Error> {
        let mut program = PythonProgram::new();
        program.write_line("import matplotlib.pyplot as plt");
        writeln!(
            &program.file,
            "plt.plot({}, {})",
            PythonLitteral(x),
            PythonLitteral(y),
        );
        program.write_line("plt.show()").run()
    }

    pub fn plot_y<Y: AsPythonLitteral>(y: &Y) -> Result<std::process::Output, std::io::Error> {
        let mut program = PythonProgram::new();
        program.write_line("import matplotlib.pyplot as plt");
        writeln!(&program.file, "plt.plot({})", PythonLitteral(y));
        program.write_line("plt.show()").run()
}
}

#[macro_export]
macro_rules! plot {
    ($y: expr) => {pycall::plots::plot_y($y)};
    ($x: expr, $y: expr) => {pycall::plots::plot_xy($x, $y)};
    ($x: expr, $y: expr, $args: expr) => {pycall::plots::plot_xyargs($x, $y, $args)};
}

#[test]
fn run() {
    let join = std::thread::spawn(|| quick_plot(&(-50..50).map(|x| (-x * x)).collect::<Vec<_>>()));
    let mut program = PythonProgram::new();
    program
        .write_line("import matplotlib.pyplot as plt")
        .define_variable(
            "hello",
            &(-50..50).map(|x| (x * x) as f64).collect::<Vec<_>>(),
        )
        .write_line("print(hello)")
        .write_line("plt.plot(hello)")
        .write_line("plt.show()");
    println!("program: {}\r\n{}", program.file.path().display(), &program);
    let output = program.run().unwrap();
    join.join();
}
