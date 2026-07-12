use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("com.hestia.app").qml_file("qml/Main.qml"))
        .qt_module("Network")
        .files(["src/cxxqt_object.rs"])
        .build();
}
