use qboy_core::QBoy;

fn main() {
    let mut boy = QBoy::new();
    boy.exec_cpu_test_rom(4);
}
