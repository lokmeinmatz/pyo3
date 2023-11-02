use pyo3::{prelude::*, types::IntoPyDict};

fn main() -> PyResult<()> {
    #[cfg(GraalPy)]
    println!("running with GraalPy interpreter");
    #[cfg(not(GraalPy))]
    println!("running with CPython interpreter");

    let res: i32 = Python::with_gil(|py| {
        let code = "a + b";

        let vars = [("a", 1), ("b", 2)].into_py_dict(py);
        let global = [("a", 1)].into_py_dict(py);
        println!("vars: {:?}", vars.get_type());
        return py.eval(code, Some(global), Some(&vars))?.extract();
    })?;

    println!("python says 1+2={}", res);
    Ok(())
}
