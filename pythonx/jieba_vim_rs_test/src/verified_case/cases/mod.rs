mod base;

pub use base::VerifiableCase;

use minijinja::Environment;
use once_cell::sync::Lazy;

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template("execute_nmap", "templates/execute_nmap.j2")
        .unwrap();
    env.add_template("execute_omap_c", "templates/execute_omap_c.j2")
        .unwrap();
    env.add_template("execute_omap_d", "templates/execute_omap_d.j2")
        .unwrap();
    env.add_template("execute_omap_y", "templates/execute_omap_y.j2")
        .unwrap();
    env.add_template("execute_xmap", "templates/execute_xmap.j2")
        .unwrap();
    env.add_template("include", "templates/include.j2").unwrap();
    env.add_template("setup_omap", "templates/setup_omap.j2")
        .unwrap();
    env.add_template("setup", "templates/setup.j2").unwrap();

    env
});
