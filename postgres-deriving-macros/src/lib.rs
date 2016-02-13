#![feature(plugin_registrar, rustc_private)]

extern crate rustc_plugin;
extern crate postgres_deriving_codegen;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut rustc_plugin::Registry) {
    postgres_deriving_codegen::register(reg);
}
