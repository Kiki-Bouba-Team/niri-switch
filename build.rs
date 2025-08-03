/* niri-switch  Copyright (C) 2025  Kiki/Bouba Team */

fn main() {
    /* Compile GTK resources. This is needed for composite templates. */
    glib_build_tools::compile_resources(
        &["src/daemon/gui/"],
        "src/daemon/gui/resources.gresource.xml",
        "composite_templates.gresource",
    );
}
