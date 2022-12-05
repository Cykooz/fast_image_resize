use std::cell::RefCell;

thread_local!(static TEST_LOGS: RefCell<Vec<String>> = RefCell::new(Vec::new()));

pub fn log_message(msg: &str) {
    TEST_LOGS.with(|f| {
        let mut logs = f.borrow_mut();
        logs.push(msg.to_string());
    });
}

pub fn logs_contain(msg: &str) -> bool {
    TEST_LOGS.with(|f| {
        let logs = f.borrow();
        for line in logs.iter() {
            if line.contains(msg) {
                return true;
            }
        }
        false
    })
}

pub fn clear_log() {
    TEST_LOGS.with(|f| {
        let mut logs = f.borrow_mut();
        logs.clear();
    })
}
