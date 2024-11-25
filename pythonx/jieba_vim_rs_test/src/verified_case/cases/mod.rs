mod base;
mod nmap_w;
mod utils;

pub use base::VerifiableCase;
pub use nmap_w::NmapWCase;

use minijinja::Environment;
use once_cell::sync::Lazy;

static TEMPLATES: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.add_template("execute_nmap", include_str!("templates/execute_nmap.j2"))
        .unwrap();
    env.add_template(
        "execute_omap_c",
        include_str!("templates/execute_omap_c.j2"),
    )
    .unwrap();
    env.add_template(
        "execute_omap_d",
        include_str!("templates/execute_omap_d.j2"),
    )
    .unwrap();
    env.add_template(
        "execute_omap_y",
        include_str!("templates/execute_omap_y.j2"),
    )
    .unwrap();
    env.add_template("execute_xmap", include_str!("templates/execute_xmap.j2"))
        .unwrap();
    env.add_template("include", include_str!("templates/include.j2"))
        .unwrap();
    env.add_template("setup_omap", include_str!("templates/setup_omap.j2"))
        .unwrap();
    env.add_template("setup", include_str!("templates/setup.j2"))
        .unwrap();

    env
});
