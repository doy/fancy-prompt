#[cfg(feature = "verbose")]
use std;

#[cfg(feature = "verbose")]
pub static mut STACK: Option<Vec<(String, std::time::Instant, std::time::Instant)>> = None;

#[cfg(feature = "verbose")]
macro_rules! start_talking_about_time {
    ($category:expr) => (
        use std;
        use verbose;
        unsafe {
            let stack = verbose::STACK.get_or_insert_with(|| Vec::new());
            let len = stack.len();
            let now = std::time::Instant::now();
            let category = $category;
            stack.push((String::from(category), now.clone(), now.clone()));
            eprintln!("{}starting {}", " ".repeat(len), category);
        }
    )
}

#[cfg(feature = "verbose")]
macro_rules! talk_about_time {
    ($what:expr) => (
        unsafe {
            let stack = verbose::STACK.get_or_insert_with(|| Vec::new());
            let len = stack.len();
            let last = stack.last_mut().unwrap();
            let elapsed = last.1.elapsed();
            eprintln!(
                "{}{}: {} took {}.{:09}s",
                " ".repeat(len),
                last.0,
                $what,
                elapsed.as_secs(),
                elapsed.subsec_nanos()
            );
            last.1 = std::time::Instant::now();
        }
    )
}

#[cfg(feature = "verbose")]
macro_rules! stop_talking_about_time {
    () => (
        unsafe {
            let stack = verbose::STACK.get_or_insert_with(|| Vec::new());
            let last = stack.pop().unwrap();
            let elapsed = last.2.elapsed();
            let len = stack.len();
            eprintln!(
                "{}ending {} (total {}.{:09}s)",
                " ".repeat(len),
                last.0,
                elapsed.as_secs(),
                elapsed.subsec_nanos()
            );
        }
    )
}

#[cfg(not(feature = "verbose"))]
macro_rules! start_talking_about_time {
    ($e:expr) => ()
}

#[cfg(not(feature = "verbose"))]
macro_rules! talk_about_time {
    ($e:expr) => ()
}

#[cfg(not(feature = "verbose"))]
macro_rules! stop_talking_about_time {
    () => ()
}
