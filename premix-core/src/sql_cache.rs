use std::sync::OnceLock;
use std::{any::TypeId, collections::HashMap, sync::Mutex};

use crate::dialect::SqlDialect;

type PlaceholderKey = (TypeId, usize);
type PlaceholderFromKey = (TypeId, usize, usize);
type PlaceholderCache = Mutex<HashMap<PlaceholderKey, &'static str>>;
type PlaceholderFromCache = Mutex<HashMap<PlaceholderFromKey, &'static str>>;

static PLACEHOLDER_CACHE: OnceLock<PlaceholderCache> = OnceLock::new();
static PLACEHOLDER_FROM_CACHE: OnceLock<PlaceholderFromCache> = OnceLock::new();

/// Returns a cached placeholder list for `(DB, count)` combinations.
pub fn cached_placeholders<DB>(count: usize) -> &'static str
where
    DB: SqlDialect + 'static,
{
    let key = (TypeId::of::<DB>(), count);
    let cache = PLACEHOLDER_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().expect("placeholder cache poisoned");
    if let Some(value) = guard.get(&key) {
        value
    } else {
        let placeholders = crate::build_placeholders::<DB>(1, count);
        let leaked: &'static str = Box::leak(placeholders.into_boxed_str());
        guard.insert(key, leaked);
        leaked
    }
}

/// Returns a cached placeholder list for `(DB, start, count)` combinations.
pub fn cached_placeholders_from<DB>(start: usize, count: usize) -> &'static str
where
    DB: SqlDialect + 'static,
{
    let key = (TypeId::of::<DB>(), start, count);
    let cache = PLACEHOLDER_FROM_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = cache.lock().expect("placeholder cache poisoned");
    if let Some(value) = guard.get(&key) {
        value
    } else {
        let placeholders = crate::build_placeholders::<DB>(start, count);
        let leaked: &'static str = Box::leak(placeholders.into_boxed_str());
        guard.insert(key, leaked);
        leaked
    }
}
