
fn calibrate_main(argc: u32, argv: *const &str) {

}

#[rpos::ctor::ctor]
fn register() {
    rpos::module::Module::register("calibrate", calibrate_main);
}