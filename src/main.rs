use kitlogger::Logger;

fn main() {
    let logger = Logger::from_config(Default::default()).unwrap();
    logger.info("Hello from Kit-Logger!");
}
