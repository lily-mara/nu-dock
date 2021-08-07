fn main() {
    nu_plugin::serve_plugin(&mut nu_dock::ContainersCommand);
}
