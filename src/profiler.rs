#[cfg(feature = "profiler")]
pub use actual::*;
#[cfg(not(feature = "profiler"))]
pub use dummy::*;

#[cfg(not(feature = "profiler"))]
mod dummy {
    use std::collections::HashMap;

    /// Get a pretty-debug-printed String of the profiling data.
    pub fn get_profiler_print() -> String {
        "profiler feature not enabled".to_string()
    }

    /// Get a clone of the profiling data at current time.
    pub fn get_profiler_clone() -> HashMap<String, String> {
        HashMap::new()
    }

    /// Get a clone of the profiling value corresponding to the given key.
    pub fn get_profiler_value_clone(_key: &str) -> Option<String> {
        None
    }

    /// Operate on a value with a given key, as an i32.
    pub fn modify_profiler_value_i32<T: FnOnce(i32) -> i32>(_key: &str, _f: T) {}

    /// Insert data into the `profiler` dictionary of datapoints.
    pub fn insert_profiling_data<T: Into<String>, U: Into<String>>(_key: T, _value: U) {}
}

#[cfg(feature = "profiler")]
mod actual {
    use std::collections::HashMap;
    use std::sync::Mutex;

    thread_local! {
        static DEBUG_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    }

    /// Get a pretty-debug-printed String of the profiling data.
    pub fn get_profiler_print() -> String {
        let mut result = None;
        DEBUG_MAP.with(|map| {
            let map = map.lock().unwrap();
            result = Some(format!("{:#?}", map));
        });
        if let Some(result) = result {
            result
        } else {
            "failed to read profiling information".to_string()
        }
    }

    /// Get a clone of the profiling data at current time.
    pub fn get_profiler_clone() -> HashMap<String, String> {
        let mut result = None;
        DEBUG_MAP.with(|map| {
            let map = map.lock().unwrap();
            result = Some(map.clone());
        });
        if let Some(result) = result {
            result
        } else {
            HashMap::new()
        }
    }

    /// Get a clone of the profiling value corresponding to the given key.
    pub fn get_profiler_value_clone(key: &str) -> Option<String> {
        let mut result = None;
        DEBUG_MAP.with(|map| {
            let map = map.lock().unwrap();
            result = map.get(key).map(|s| s.to_string());
        });
        result
    }

    /// Operate on a value with a given key, as an i32.
    pub fn modify_profiler_value_i32<T: FnOnce(i32) -> i32>(key: &str, f: T) {
        DEBUG_MAP.with(|map| {
            let mut map = map.lock().unwrap();
            if let Some(new_value) = map
                .get(key)
                .and_then(|s| i32::from_str_radix(s, 10).ok())
                .map(f)
                .map(|i| i.to_string())
            {
                map.insert(key.to_string(), new_value);
            }
        });
    }

    /// Insert data into the `profiler` dictionary of datapoints.
    pub fn insert_profiling_data<T: Into<String>, U: Into<String>>(key: T, value: U) {
        let (key, value) = (key.into(), value.into());
        DEBUG_MAP.with(move |map| {
            let mut map = map.lock().unwrap();
            map.insert(key, value);
        });
    }
}
