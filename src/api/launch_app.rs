use anyhow::Result;
use pyo3::prelude::*;
use pyo3::{py_run, PyCell};

#[pyclass]
struct AppArgs {
    #[pyo3(get, set)]
    port: i32,

    #[pyo3(get, set)]
    login: bool,
}

pub fn launch_app(port: i32, login: bool) -> Result<(), anyhow::Error> {
    Python::with_gil(|py| {
        let app_args = AppArgs { port, login };
        let app_args = PyCell::new(py, app_args).unwrap();
        py_run!(
            py,
            app_args,
            r#"
            from opsml.app.main import OpsmlApp

            model_api = OpsmlApp(port=app_args.port, login=app_args.login)
            model_api.build_app()
            model_api.run()

            "#
        );
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_launch_app() {
        // spawn new thread
        thread::spawn(|| {
            let port = 8000;
            let login = false;

            launch_app(port, login).unwrap();
        });

        thread::sleep(Duration::from_secs(15));

        //shut down the app on port 8000
    }
}
