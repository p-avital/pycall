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

as_py_lit_impl!(str, "\"\"\"{}\"\"\"");
as_py_lit_impl!(String, "\"\"\"{}\"\"\"");
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

impl AsPythonLitteral for f32 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.is_nan() {
            write!(f, "float('nan')")
        } else {
            write!(f, "{:.6e}", &self)
        }
    }
}

impl AsPythonLitteral for f64 {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.is_nan() {
            write!(f, "float('nan')")
        } else {
            write!(f, "{:.6e}", &self)
        }
    }
}

impl<T: AsPythonLitteral> AsPythonLitteral for [T] {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[")?;
        for x in self.iter() {
            write!(f, "{},", PythonLiteral(x))?;
        }
        write!(f, "]")
    }
}

impl<T: AsPythonLitteral> AsPythonLitteral for Vec<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "[")?;
        for x in self.iter() {
            write!(f, "{},", PythonLiteral(x))?;
        }
        write!(f, "]")
    }
}

impl<K: AsPythonLitteral, V: AsPythonLitteral> AsPythonLitteral
    for std::collections::HashMap<K, V>
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{")?;
        for (k, v) in self.iter() {
            write!(f, "{}:{},", PythonLiteral(k), PythonLiteral(v))?;
        }
        write!(f, "}}")
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

struct PythonLiteral<'l, T: AsPythonLitteral + ?Sized>(pub &'l T);
impl<'l, T: AsPythonLitteral + ?Sized> Display for PythonLiteral<'l, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.0.fmt(f)
    }
}

pub struct JoinGuard<T>(Option<std::thread::JoinHandle<T>>);

impl<T> JoinGuard<T> {
    pub fn new() -> Self {
        JoinGuard(None)
    }

    pub fn spawn<F: FnOnce() -> T>(f: F) -> Self
    where
        T: Send + 'static,
        F: Send + 'static,
    {
        JoinGuard(Some(std::thread::spawn(f)))
    }

    pub fn join(mut self) -> Result<T, Box<dyn std::any::Any + Send>>
    where
        T: std::any::Any + Send + 'static,
    {
        self.0.take().unwrap().join()
    }

    pub fn detach(mut self) -> Option<std::thread::JoinHandle<T>> {
        self.0.take()
    }
}

impl<T> Drop for JoinGuard<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.join();
        }
    }
}

/// An instance of code generation unit.
/// It really is just a file with dedicated APIs to write Python into it.
/// Most importantly: it manages indentation for you.
pub struct PythonProgram {
    file: tempfile::NamedTempFile,
    indents: Indents,
}
impl PythonProgram {
    /// Creates a named temp file to store the generated python program
    pub fn new() -> PythonProgram {
        PythonProgram {
            file: tempfile::NamedTempFile::new().unwrap(),
            indents: Indents(0),
        }
    }

    pub fn save_as<P: AsRef<std::path::Path>>(&self, path: P) -> Result<u64, std::io::Error> {
        std::fs::copy(self.file.path(), path)
    }

    /// Runs the program using python3
    pub fn run(&self) -> Result<std::process::Output, std::io::Error> {
        std::process::Command::new("python3")
            .arg(self.file.path())
            .output()
    }

    /// Spawns a thread to run the program using python3.
    /// The returned JoinGuard ensures that the program will be ran to completion.
    pub fn background_run(self) -> JoinGuard<Result<std::process::Output, std::io::Error>> {
        JoinGuard::spawn(move || self.run())
    }

    /// Ensures that the internal file has been flushed. Typically not necessary.
    pub fn flush(&mut self) -> &mut Self {
        self.file.flush().unwrap();
        self
    }

    /// Moves the indentation level by `n`. However, I recommend using the dedicated functions when possible/
    pub fn indent(&mut self, n: isize) -> &mut Self {
        if n >= 0 {
            self.indents.0 += n as isize
        } else {
            self.indents.0 -= n as isize
        }
        self
    }

    /// Removes one indentation level from the cursor.
    /// You should call this whenever you're done with a scope.
    pub fn end_block(&mut self) -> &mut Self {
        self.indent(-1)
    }

    /// Writes a line assigning `value` formatted as a python literal to `name`
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
            PythonLiteral(value)
        )
        .unwrap();
        self
    }

    /// Writes an import statement for your `dependency`
    pub fn import(&mut self, dependency: &str) -> &mut Self {
        writeln!(&mut self.file, "{}import {}", self.indents, dependency).unwrap();
        self
    }

    /// Writes an import statement for your `dependency` as `rename`
    pub fn import_as(&mut self, dependency: &str, rename: &str) -> &mut Self {
        writeln!(
            &mut self.file,
            "{}import {} as {}",
            self.indents, dependency, rename
        )
        .unwrap();
        self
    }

    /// Writes whatever line you passed it, indented at the proper level.
    pub fn write_line(&mut self, line: &str) -> &mut Self {
        writeln!(&mut self.file, "{}{}", self.indents, line).unwrap();
        self
    }

    /// Writes an if, using your condition as a test, and increments indentation.
    pub fn r#if(&mut self, condition: &str) -> &mut Self {
        writeln!(&mut self.file, "{}if {}:", self.indents, condition).unwrap();
        self.indent(1)
    }
    /// Decrements indentation, writes an elif, using your condition as a test, and increments indentation.
    pub fn elif(&mut self, condition: &str) -> &mut Self {
        self.indent(-1);
        writeln!(&mut self.file, "{}elif {}:", self.indents, condition).unwrap();
        self.indent(1)
    }
    /// Decrements indentation, writes an else, using your condition as a test, and increments indentation.
    pub fn r#else(&mut self) -> &mut Self {
        self.indent(-1).write_line("else:").indent(1)
    }

    /// Writes "for `range`:", and increments indentation.
    pub fn r#for(&mut self, range: &str) -> &mut Self {
        writeln!(&mut self.file, "{}for {}:", self.indents, range).unwrap();
        self.indent(1)
    }

    /// Writes a while, using your condition as a test, and increments indentation.
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

pub trait MatPlotLib {
    fn import_pyplot_as_plt(&mut self) -> &mut Self;
    fn plot_y<Y: AsPythonLitteral>(&mut self, y: &Y) -> &mut Self;
    fn plot_xy<X: AsPythonLitteral, Y: AsPythonLitteral>(&mut self, x: &X, y: &Y) -> &mut Self;
    fn plot_xyargs<X: AsPythonLitteral, Y: AsPythonLitteral>(
        &mut self,
        x: &X,
        y: &Y,
        args: &str,
    ) -> &mut Self;
    fn semilogy_y<Y: AsPythonLitteral>(&mut self, y: &Y) -> &mut Self;
    fn semilogy_xy<X: AsPythonLitteral, Y: AsPythonLitteral>(&mut self, x: &X, y: &Y) -> &mut Self;
    fn semilogy_xyargs<X: AsPythonLitteral, Y: AsPythonLitteral>(
        &mut self,
        x: &X,
        y: &Y,
        args: &str,
    ) -> &mut Self;
    fn show(&mut self) -> &mut Self;
}

impl MatPlotLib for PythonProgram {
    fn import_pyplot_as_plt(&mut self) -> &mut Self {
        self.import_as("matplotlib.pyplot", "plt")
    }

    fn plot_y<Y: AsPythonLitteral>(&mut self, y: &Y) -> &mut Self {
        self.write_line(&format!("plt.plot({})", PythonLiteral(y)))
    }

    fn plot_xy<X: AsPythonLitteral, Y: AsPythonLitteral>(&mut self, x: &X, y: &Y) -> &mut Self {
        self.write_line(&format!(
            "plt.plot({},{})",
            PythonLiteral(x),
            PythonLiteral(y)
        ))
    }

    fn plot_xyargs<X: AsPythonLitteral, Y: AsPythonLitteral>(
        &mut self,
        x: &X,
        y: &Y,
        args: &str,
    ) -> &mut Self {
        self.write_line(&format!(
            "plt.plot({},{},{})",
            PythonLiteral(x),
            PythonLiteral(y),
            args
        ))
    }

    fn semilogy_y<Y: AsPythonLitteral>(&mut self, y: &Y) -> &mut Self {
        self.write_line(&format!("plt.semilogy({})", PythonLiteral(y)))
    }

    fn semilogy_xy<X: AsPythonLitteral, Y: AsPythonLitteral>(&mut self, x: &X, y: &Y) -> &mut Self {
        self.write_line(&format!(
            "plt.semilogy({},{})",
            PythonLiteral(x),
            PythonLiteral(y)
        ))
    }

    fn semilogy_xyargs<X: AsPythonLitteral, Y: AsPythonLitteral>(
        &mut self,
        x: &X,
        y: &Y,
        args: &str,
    ) -> &mut Self {
        self.write_line(&format!(
            "plt.semilogy({},{},{})",
            PythonLiteral(x),
            PythonLiteral(y),
            args
        ))
    }

    fn show(&mut self) -> &mut Self {
        self.write_line("plt.show()")
    }
}

pub mod plots {
    use crate::{AsPythonLitteral, PythonLiteral, PythonProgram};
    use std::io::Write;

    pub fn plot_xyargs<X: AsPythonLitteral, Y: AsPythonLitteral>(
        x: &X,
        y: &Y,
        args: &str,
    ) -> Result<std::process::Output, std::io::Error> {
        let mut program = PythonProgram::new();
        program.import_as("matplotlib.pyplot", "plt");
        writeln!(
            &program.file,
            "plt.plot({}, {}, {})",
            PythonLiteral(x),
            PythonLiteral(y),
            PythonLiteral(args)
        );
        program.write_line("plt.show()").run()
    }

    pub fn plot_xy<X: AsPythonLitteral, Y: AsPythonLitteral>(
        x: &X,
        y: &Y,
    ) -> Result<std::process::Output, std::io::Error> {
        let mut program = PythonProgram::new();
        program.import_as("matplotlib.pyplot", "plt");
        writeln!(
            &program.file,
            "plt.plot({}, {})",
            PythonLiteral(x),
            PythonLiteral(y),
        );
        program.write_line("plt.show()").run()
    }

    pub fn plot_y<Y: AsPythonLitteral>(y: &Y) -> Result<std::process::Output, std::io::Error> {
        let mut program = PythonProgram::new();
        program.import_as("matplotlib.pyplot", "plt");
        writeln!(&program.file, "plt.plot({})", PythonLiteral(y));
        program.write_line("plt.show()").run()
    }
}

#[macro_export]
macro_rules! plot {
    ($y: expr) => {
        pycall::plots::plot_y($y)
    };
    ($x: expr, $y: expr) => {
        pycall::plots::plot_xy($x, $y)
    };
    ($x: expr, $y: expr, $args: expr) => {
        pycall::plots::plot_xyargs($x, $y, $args)
    };
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
