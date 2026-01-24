use once_cell::sync::OnceLock;
use std::{any::TypeId, collections::HashMap, sync::Mutex};

use crate::dialect::SqlDialect;

static PLACEHOLDER_CACHE: OnceLock<Mutex<HashMap<(TypeId, usize), Box<str>>>> = OnceLock::new();

/// Returns a cached placeholder list for `(DB, count)` combinations.
pub fn cached_placeholders<DB>(count: usize) -> &'static str
where
    DB: SqlDialect + 'static,
{
    let key = (TypeId::of::<DB>(), count);
    let cache = PLACEHOLDER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().expect("placeholder cache poisoned");
    if let Some(value) = guard.get(&key) {
        value.as_ref()
    } else {
        let placeholders = crate::build_placeholders::<DB>(1, count);
        let boxed = placeholders.into_boxed_str();
        let entry = guard.entry(key).or_insert(boxed);
        entry.as_ref()
    }
}
